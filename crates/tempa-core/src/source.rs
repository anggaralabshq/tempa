//! Load the canonical source: `AGENTS.md` + `.tempa/`.

use crate::config::Config;
use crate::md::{Front, split_front};
use crate::{Error, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

pub const TEMPA_DIR: &str = ".tempa";

/// A command or subagent definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Path relative to its kind dir, `/`-separated, e.g. `db/migrate.md`.
    pub rel: String,
    pub front: Option<Front>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub name: String,
    /// Files relative to the skill dir.
    pub files: Vec<(String, Vec<u8>)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct McpServer {
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Source {
    pub agents_md: String,
    pub commands: Vec<Item>,
    pub agents: Vec<Item>,
    pub skills: Vec<Skill>,
    pub mcp: BTreeMap<String, McpServer>,
    /// `.tempa/targets/claude.md`, appended to `CLAUDE.md`.
    pub claude_extra: Option<String>,
    /// `.tempa/targets/<output path>` base files, keyed by output path.
    pub bases: BTreeMap<String, Vec<u8>>,
}

#[derive(Deserialize, Default)]
struct McpFile {
    #[serde(default)]
    servers: BTreeMap<String, McpServer>,
}

pub fn parse_item(rel: String, text: &str) -> Item {
    let (front, body) = split_front(text);
    Item { rel, front, body: crate::md::normalize_eol(body) }
}

pub fn front_get<'a>(item: &'a Item, key: &str) -> Option<&'a str> {
    item.front.as_ref()?.pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// All files under `dir`, as (`/`-separated relative path, full path), sorted.
pub fn walk(dir: &Path) -> Result<Vec<(String, std::path::PathBuf)>> {
    fn go(base: &Path, dir: &Path, out: &mut Vec<(String, std::path::PathBuf)>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                go(base, &path, out)?;
            } else {
                let rel = path.strip_prefix(base).expect("under base");
                let rel = rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/");
                out.push((rel, path));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    if dir.is_dir() {
        go(dir, dir, &mut out)?;
    }
    out.sort();
    Ok(out)
}

fn items(dir: &Path) -> Result<Vec<Item>> {
    walk(dir)?
        .into_iter()
        .filter(|(rel, _)| rel.ends_with(".md"))
        .map(|(rel, path)| Ok(parse_item(rel, &std::fs::read_to_string(path)?)))
        .collect()
}

pub fn load(root: &Path, cfg: &Config) -> Result<Source> {
    let t = root.join(TEMPA_DIR);
    let agents_md = std::fs::read_to_string(root.join(&cfg.docs.context)).unwrap_or_default();

    let mut skills: BTreeMap<String, Skill> = BTreeMap::new();
    for (rel, path) in walk(&t.join("skills"))? {
        let Some((name, file)) = rel.split_once('/') else { continue };
        skills
            .entry(name.to_string())
            .or_insert_with(|| Skill { name: name.to_string(), files: Vec::new() })
            .files
            .push((file.to_string(), std::fs::read(path)?));
    }

    let mcp = match std::fs::read_to_string(t.join("mcp.toml")) {
        Ok(text) => toml::from_str::<McpFile>(&text)
            .map_err(|e| Error(format!(".tempa/mcp.toml tidak valid: {e}")))?
            .servers,
        Err(_) => BTreeMap::new(),
    };

    let mut claude_extra = None;
    let mut bases = BTreeMap::new();
    for (rel, path) in walk(&t.join("targets"))? {
        if rel == "claude.md" {
            claude_extra = Some(crate::md::normalize_eol(&std::fs::read_to_string(path)?));
        } else {
            bases.insert(rel, std::fs::read(path)?);
        }
    }

    Ok(Source {
        agents_md,
        commands: items(&t.join("commands"))?,
        agents: items(&t.join("agents"))?,
        skills: skills.into_values().collect(),
        mcp,
        claude_extra,
        bases,
    })
}
