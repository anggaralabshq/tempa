//! `tempa.toml`: load, defaults, and format-preserving edits.

use crate::{Error, Result};
use serde::Deserialize;
use std::path::Path;

pub const FILE: &str = "tempa.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    Claude,
    Codex,
    Gemini,
    Opencode,
    Cursor,
    Copilot,
}

impl Agent {
    pub const ALL: [Agent; 6] = [
        Agent::Claude,
        Agent::Codex,
        Agent::Gemini,
        Agent::Opencode,
        Agent::Cursor,
        Agent::Copilot,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Agent::Claude => "claude",
            Agent::Codex => "codex",
            Agent::Gemini => "gemini",
            Agent::Opencode => "opencode",
            Agent::Cursor => "cursor",
            Agent::Copilot => "copilot",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub tempa_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Docs {
    pub context: String,
    pub plan: String,
    pub decisions: String,
    pub changes: String,
    pub harness: String,
}

impl Default for Docs {
    fn default() -> Self {
        Docs {
            context: "AGENTS.md".into(),
            plan: "docs/plans/".into(),
            decisions: "docs/decisions/".into(),
            changes: "CHANGES.md".into(),
            harness: "HARNESS.md".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Headings {
    pub context: Vec<String>,
    pub plan: Vec<String>,
    pub decisions: Vec<String>,
    pub harness: Vec<String>,
}

fn strings(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

impl Default for Headings {
    fn default() -> Self {
        Headings {
            context: strings(&["Stack", "Commands", "Conventions", "Off-limits"]),
            plan: strings(&["Goal", "Steps", "Files touched", "Risks"]),
            decisions: strings(&[
                "Context",
                "Options",
                "Decision",
                "Consequences",
                "Agent + model",
            ]),
            harness: strings(&["Definition of done", "Gates"]),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Agents {
    pub enabled: Vec<Agent>,
}

impl Default for Agents {
    fn default() -> Self {
        Agents {
            enabled: Agent::ALL.to_vec(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Gates {
    pub plan_before_edit: bool,
    pub review_required: bool,
    pub decision_required_for: Vec<String>,
    pub schema_paths: Vec<String>,
    pub public_api_paths: Vec<String>,
    pub max_diff_lines: u32,
    pub protected_paths: Vec<String>,
}

impl Default for Gates {
    fn default() -> Self {
        Gates {
            plan_before_edit: true,
            review_required: false,
            decision_required_for: strings(&["new-dependency", "schema-change", "public-api"]),
            schema_paths: vec![],
            public_api_paths: vec![],
            max_diff_lines: 800,
            protected_paths: strings(&[".env*", "tempa.toml"]),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Ci {
    pub base: String,
    pub skip_authors: Vec<String>,
    pub docs_only_paths: Vec<String>,
}

impl Default for Ci {
    fn default() -> Self {
        Ci {
            base: "main".into(),
            skip_authors: strings(&["renovate[bot]"]),
            docs_only_paths: strings(&["docs/**", "*.md"]),
        }
    }
}

/// `[mcp]`: the `tempa mcp` server tempa adds to every agent's MCP config.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Mcp {
    pub serve: bool,
}

impl Default for Mcp {
    fn default() -> Self {
        Mcp { serve: true }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub project: Project,
    pub docs: Docs,
    pub headings: Headings,
    pub agents: Agents,
    /// Harness commands in file order.
    pub harness: Vec<(String, String)>,
    pub gates: Gates,
    pub ci: Ci,
    pub mcp: Mcp,
}

#[derive(Deserialize)]
struct Raw {
    project: Project,
    #[serde(default)]
    docs: Docs,
    #[serde(default)]
    headings: Headings,
    #[serde(default)]
    agents: Agents,
    #[serde(default)]
    harness: toml::Table,
    #[serde(default)]
    gates: Gates,
    #[serde(default)]
    ci: Ci,
    #[serde(default)]
    mcp: Mcp,
}

impl Config {
    pub fn load(root: &Path) -> Result<Config> {
        let path = root.join(FILE);
        let text = std::fs::read_to_string(&path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Error(format!("{FILE} not found; run `tempa init`")),
            _ => Error(format!("{FILE} could not be read: {e}")),
        })?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Config> {
        let raw: Raw =
            toml::from_str(text).map_err(|e| Error(format!("{FILE} is invalid: {e}")))?;
        let mut harness = Vec::new();
        for (name, value) in raw.harness {
            let cmd = value.as_str().ok_or_else(|| {
                Error(format!("{FILE}: [harness].{name} must be a command string"))
            })?;
            harness.push((name, cmd.to_string()));
        }
        Ok(Config {
            project: raw.project,
            docs: raw.docs,
            headings: raw.headings,
            agents: raw.agents,
            harness,
            gates: raw.gates,
            ci: raw.ci,
            mcp: raw.mcp,
        })
    }
}

/// Default `tempa.toml` text (spec 9.1).
pub fn default_toml(project_name: &str) -> String {
    let project_name = toml::Value::String(project_name.to_string()).to_string();
    format!(
        r#"# Tempa configuration. The single source of project rules.
[project]
name = {project_name}
tempa_version = "{version}"

[docs]            # role -> file name, rename freely
context   = "AGENTS.md"
plan      = "docs/plans/"
decisions = "docs/decisions/"
changes   = "CHANGES.md"
harness   = "HARNESS.md"

[agents]
enabled = ["claude", "codex", "gemini", "opencode", "cursor", "copilot"]

[harness]         # run in order; every command must exit 0
# format    = "bun run format:check"
# lint = "bun run lint"
# typecheck = "bun run typecheck"
# test      = "bun run test"

[gates]
plan_before_edit      = true
review_required       = false
decision_required_for = ["new-dependency", "schema-change", "public-api"]
schema_paths          = []   # e.g. ["migrations/**"]
public_api_paths      = []   # e.g. ["src/api/**"]
max_diff_lines        = 800
protected_paths       = [".env*", "tempa.toml"]   # add your CI files, e.g. "bitbucket-pipelines.yml" or ".github/workflows/**"

[ci]
base            = "main"
skip_authors    = ["renovate[bot]"]
docs_only_paths = ["docs/**", "*.md"]

[mcp]
serve = true      # add the `tempa mcp` server to every agent's MCP config
"#,
        version = env!("CARGO_PKG_VERSION")
    )
}

fn edit(root: &Path, f: impl FnOnce(&mut toml_edit::DocumentMut) -> Result<bool>) -> Result<bool> {
    let path = root.join(FILE);
    let mut doc: toml_edit::DocumentMut = std::fs::read_to_string(&path)?.parse()?;
    let changed = f(&mut doc)?;
    if changed {
        crate::write_atomic(&path, doc.to_string().as_bytes())?;
    }
    Ok(changed)
}

/// Add `[harness].<name> = <cmd>` unless `name` exists. Returns whether it changed.
pub fn add_harness(root: &Path, name: &str, cmd: &str) -> Result<bool> {
    edit(root, |doc| {
        let table = doc
            .entry("harness")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .ok_or_else(|| Error(format!("{FILE}: harness must be a table")))?;
        if table.contains_key(name) {
            return Ok(false);
        }
        table.insert(name, toml_edit::value(cmd));
        Ok(true)
    })
}

/// Append `glob` to `[gates].protected_paths` unless present. Returns whether it changed.
pub fn add_protected(root: &Path, glob: &str) -> Result<bool> {
    edit(root, |doc| {
        let gates = doc
            .entry("gates")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .ok_or_else(|| Error(format!("{FILE}: gates must be a table")))?;
        if !gates.contains_key("protected_paths") {
            let defaults: toml_edit::Array = Gates::default().protected_paths.iter().collect();
            gates.insert("protected_paths", toml_edit::value(defaults));
        }
        let arr = gates["protected_paths"]
            .as_array_mut()
            .ok_or_else(|| Error(format!("{FILE}: gates.protected_paths must be an array")))?;
        if arr.iter().any(|v| v.as_str() == Some(glob)) {
            return Ok(false);
        }
        arr.push(glob);
        Ok(true)
    })
}
