//! One ACP session on a connection: handshake with the tempa MCP server injected,
//! prompts, cancel, model choice.

use super::conn::Conn;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The injected MCP server (`tempa-app --mcp …`).
pub struct McpLaunch {
    pub command: PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelVia {
    /// `session/set_config_option` with this config id (stable ACP).
    ConfigOption(String),
    /// `session/set_model` (unstable ACP, some adapters).
    SetModel,
    None,
}

#[derive(Debug, Clone)]
pub struct Models {
    pub current: Option<String>,
    /// (id, name).
    pub available: Vec<(String, String)>,
    pub via: ModelVia,
}

/// Model choices from a `session/new` result: a config option with category `model`
/// first, the unstable `models` field otherwise.
pub fn models(new: &Value) -> Models {
    let opt = new["configOptions"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|o| o["category"] == "model" && o["type"] == "select");
    if let Some(o) = opt {
        let mut available = Vec::new();
        for item in o["options"].as_array().into_iter().flatten() {
            // Flat options, or groups holding options.
            let inner = item["options"]
                .as_array()
                .cloned()
                .unwrap_or_else(|| vec![item.clone()]);
            for x in inner {
                if let (Some(v), Some(n)) = (x["value"].as_str(), x["name"].as_str()) {
                    available.push((v.to_string(), n.to_string()));
                }
            }
        }
        return Models {
            current: o["currentValue"].as_str().map(String::from),
            available,
            via: ModelVia::ConfigOption(o["id"].as_str().unwrap_or_default().to_string()),
        };
    }
    if let Some(list) = new["models"]["availableModels"].as_array() {
        return Models {
            current: new["models"]["currentModelId"].as_str().map(String::from),
            available: list
                .iter()
                .filter_map(|m| {
                    Some((
                        m["modelId"].as_str()?.to_string(),
                        m["name"].as_str()?.to_string(),
                    ))
                })
                .collect(),
            via: ModelVia::SetModel,
        };
    }
    Models {
        current: None,
        available: vec![],
        via: ModelVia::None,
    }
}

pub struct Session {
    conn: Conn,
    id: String,
    models: Models,
}

const NEW_SESSION: Duration = Duration::from_secs(30);

impl Session {
    /// `initialize` (waits up to `init_timeout`: a first `npx -y` downloads the
    /// adapter) then `session/new` in `cwd` with the tempa MCP server.
    pub fn start(
        conn: Conn,
        cwd: &Path,
        mcp: &McpLaunch,
        init_timeout: Duration,
    ) -> Result<Session, String> {
        conn.request(
            "initialize",
            json!({
                "protocolVersion": 1,
                "clientCapabilities": {"fs": {"readTextFile": false, "writeTextFile": false}, "terminal": false},
                "clientInfo": {"name": "tempa", "version": env!("CARGO_PKG_VERSION")},
            }),
            Some(init_timeout),
        )?;
        let new = conn.request(
            "session/new",
            json!({
                "cwd": cwd,
                "mcpServers": [{"name": "tempa", "command": mcp.command, "args": mcp.args, "env": []}],
            }),
            Some(NEW_SESSION),
        )?;
        let id = new["sessionId"]
            .as_str()
            .ok_or("agent did not return a session id")?
            .to_string();
        Ok(Session {
            conn,
            id,
            models: models(&new),
        })
    }

    #[cfg(test)]
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn models(&self) -> &Models {
        &self.models
    }

    /// Run one turn; returns the stop reason.
    pub fn prompt(&self, text: &str) -> Result<String, String> {
        let r = self.conn.request(
            "session/prompt",
            json!({"sessionId": self.id, "prompt": [{"type": "text", "text": text}]}),
            None,
        )?;
        Ok(r["stopReason"].as_str().unwrap_or("end_turn").to_string())
    }

    /// End the agent process tree now (a running prompt fails with "agent exited").
    pub fn stop(&self) {
        self.conn.stop();
    }

    pub fn cancel(&self) {
        let _ = self
            .conn
            .notify("session/cancel", json!({"sessionId": self.id}));
    }

    pub fn set_model(&self, model: &str) -> Result<(), String> {
        match &self.models.via {
            ModelVia::ConfigOption(cid) => self.conn.request(
                "session/set_config_option",
                json!({"sessionId": self.id, "configId": cid, "value": model}),
                Some(NEW_SESSION),
            ),
            ModelVia::SetModel => self.conn.request(
                "session/set_model",
                json!({"sessionId": self.id, "modelId": model}),
                Some(NEW_SESSION),
            ),
            ModelVia::None => return Err("this agent does not offer a model choice".into()),
        }
        .map(|_| ())
    }
}
