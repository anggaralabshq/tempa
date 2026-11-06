//! Live chat sessions for the UI: one agent process per chat, events out through
//! `emit`, permission requests waiting on the user's answer.

use super::conn::{Conn, Handlers};
use super::session::{McpLaunch, Session};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

pub type Emit = Arc<dyn Fn(Value) + Send + Sync>;
type Waiting = Arc<Mutex<HashMap<u64, mpsc::Sender<Option<String>>>>>;

struct Chat {
    session: Session,
    waiting: Waiting,
    emit: Emit,
}

impl Chat {
    /// Every open permission request answers `cancelled` (ACP requires it on cancel).
    fn cancel_waiting(&self) {
        for (_, tx) in self
            .waiting
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .drain()
        {
            let _ = tx.send(None);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Started {
    pub id: u32,
    pub current_model: Option<String>,
    /// (id, name).
    pub models: Vec<(String, String)>,
}

#[derive(Default)]
pub struct ChatManager {
    next: AtomicU32,
    chats: Mutex<HashMap<u32, Arc<Chat>>>,
}

/// First `npx -y` run downloads the adapter.
const INIT_TIMEOUT: Duration = Duration::from_secs(120);

impl ChatManager {
    fn get(&self, id: u32) -> Result<Arc<Chat>, String> {
        self.chats
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&id)
            .cloned()
            .ok_or_else(|| "chat has ended".into())
    }

    /// Start `program args` in `cwd` as an ACP agent with the tempa MCP server.
    /// Events: `update`, `permission`, `turn_end`, `error`, `exit`.
    pub fn start_with(
        &self,
        program: &Path,
        args: &[String],
        cwd: &Path,
        path_env: Option<&str>,
        mcp: McpLaunch,
        emit: Emit,
    ) -> Result<Started, String> {
        let waiting: Waiting = Arc::default();
        let next_req = Arc::new(AtomicU64::new(1));
        let (on_update, on_ask, on_exit) = (emit.clone(), emit.clone(), emit.clone());
        let w = Arc::clone(&waiting);
        let handlers = Handlers {
            notify: Box::new(move |method: &str, p: Value| {
                if method == "session/update" {
                    on_update(json!({"type": "update", "update": p["update"]}));
                }
            }),
            request: Box::new(move |method: &str, p: Value| {
                if method != "session/request_permission" {
                    return Err(
                        json!({"code": -32601, "message": format!("{method} is not supported by tempa")}),
                    );
                }
                let request = next_req.fetch_add(1, Ordering::Relaxed);
                let (tx, rx) = mpsc::channel();
                w.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .insert(request, tx);
                on_ask(
                    json!({"type": "permission", "request": request, "toolCall": p["toolCall"], "options": p["options"]}),
                );
                Ok(match rx.recv().ok().flatten() {
                    Some(option) => json!({"outcome": {"outcome": "selected", "optionId": option}}),
                    None => json!({"outcome": {"outcome": "cancelled"}}),
                })
            }),
            exit: Box::new(move |tail: String| {
                on_exit(json!({"type": "exit", "stderr": tail}));
            }),
        };
        let conn = Conn::spawn(program, args, cwd, path_env, &[], handlers)?;
        let session = Session::start(conn, cwd, &mcp, INIT_TIMEOUT)?;
        let started = Started {
            id: self.next.fetch_add(1, Ordering::Relaxed) + 1,
            current_model: session.models().current.clone(),
            models: session.models().available.clone(),
        };
        let chat = Arc::new(Chat {
            session,
            waiting,
            emit,
        });
        self.chats
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(started.id, chat);
        Ok(started)
    }

    /// Send a turn; the result arrives as `turn_end` or `error`.
    pub fn prompt(&self, id: u32, text: &str) -> Result<(), String> {
        let chat = self.get(id)?;
        let text = text.to_string();
        std::thread::spawn(move || {
            let ev = match chat.session.prompt(&text) {
                Ok(reason) => json!({"type": "turn_end", "stopReason": reason}),
                Err(message) => json!({"type": "error", "message": message}),
            };
            (chat.emit)(ev);
        });
        Ok(())
    }

    pub fn cancel(&self, id: u32) {
        if let Ok(chat) = self.get(id) {
            chat.cancel_waiting();
            chat.session.cancel();
        }
    }

    pub fn answer(&self, id: u32, request: u64, option: Option<String>) -> Result<(), String> {
        let chat = self.get(id)?;
        let tx = chat
            .waiting
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&request)
            .ok_or("this permission request was already answered")?;
        let _ = tx.send(option);
        Ok(())
    }

    pub fn set_model(&self, id: u32, model: &str) -> Result<(), String> {
        self.get(id)?.session.set_model(model)
    }

    /// End the chat: cancel the turn, answer what waits, stop the agent tree.
    pub fn stop(&self, id: u32) {
        let chat = self
            .chats
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&id);
        if let Some(chat) = chat {
            chat.cancel_waiting();
            chat.session.cancel();
            // Not on the last Arc drop: a running prompt thread still holds one.
            chat.session.stop();
        }
    }

    pub fn stop_all(&self) {
        let ids: Vec<u32> = self
            .chats
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .keys()
            .copied()
            .collect();
        for id in ids {
            self.stop(id);
        }
    }
}
