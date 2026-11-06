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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// HTTP headers for remote servers (`${VAR}` interpolation, Claude syntax).
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
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

#[derive(Deserialize, serde::Serialize, Default)]
pub struct McpFile {
    #[serde(default)]
    pub servers: BTreeMap<String, McpServer>,
}

pub fn parse_item(rel: String, text: &str) -> Item {
    let (front, body) = split_front(text);
    Item {
        rel,
        front,
        body: crate::md::normalize_eol(body),
    }
}

pub fn front_get<'a>(item: &'a Item, key: &str) -> Option<&'a str> {
    item.front
        .as_ref()?
        .pairs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// All files under `dir`, as (`/`-separated relative path, full path), sorted.
/// Symlinked dirs are followed (up to a fixed depth, so a loop is an error, not a hang);
/// callers that copy content must check [`inside`].
pub fn walk(dir: &Path) -> Result<Vec<(String, std::path::PathBuf)>> {
    fn go(
        base: &Path,
        dir: &Path,
        depth: usize,
        out: &mut Vec<(String, std::path::PathBuf)>,
    ) -> Result<()> {
        if depth > 32 {
            return Err(Error(format!(
                "{}: folder terlalu dalam (symlink berputar?)",
                dir.display()
            )));
        }
        for entry in std::fs::read_dir(dir).map_err(|e| Error(format!("{}: {e}", dir.display())))? {
            let path = entry?.path();
            if std::fs::metadata(&path).is_ok_and(|m| m.is_dir()) {
                go(base, &path, depth + 1, out)?;
            } else {
                let rel = path.strip_prefix(base).expect("under base");
                let rel = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push((rel, path));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    if dir.is_dir() {
        go(dir, dir, 0, &mut out)?;
    }
    out.sort();
    Ok(out)
}

/// `path` resolves (symlinks included) to a location inside `root`.
pub fn inside(root: &Path, path: &Path) -> bool {
    match (std::fs::canonicalize(root), std::fs::canonicalize(path)) {
        (Ok(r), Ok(p)) => p.starts_with(r),
        _ => false,
    }
}

/// Walk a `.tempa/` dir; a symlink out of the repo would publish personal files.
fn walk_source(root: &Path, dir: &Path) -> Result<Vec<(String, std::path::PathBuf)>> {
    let files = walk(dir)?;
    if let Some((_, p)) = files.iter().find(|(_, p)| !inside(root, p)) {
        return Err(Error(format!(
            "{}: symlink ke luar repo tidak diizinkan di .tempa/",
            p.display()
        )));
    }
    Ok(files)
}

fn items(root: &Path, dir: &Path) -> Result<Vec<Item>> {
    walk_source(root, dir)?
        .into_iter()
        .filter(|(rel, _)| rel.ends_with(".md"))
        .map(|(rel, path)| Ok(parse_item(rel, &std::fs::read_to_string(path)?)))
        .collect()
}

pub fn load(root: &Path, cfg: &Config) -> Result<Source> {
    let t = root.join(TEMPA_DIR);
    let agents_md = std::fs::read_to_string(root.join(&cfg.docs.context)).unwrap_or_default();

    let mut skills: BTreeMap<String, Skill> = BTreeMap::new();
    for (rel, path) in walk_source(root, &t.join("skills"))? {
        let Some((name, file)) = rel.split_once('/') else {
            continue;
        };
        skills
            .entry(name.to_string())
            .or_insert_with(|| Skill {
                name: name.to_string(),
                files: Vec::new(),
            })
            .files
            .push((file.to_string(), std::fs::read(path)?));
    }

    let mcp = match std::fs::read_to_string(t.join("mcp.toml")) {
        Ok(text) => {
            toml::from_str::<McpFile>(&text)
                .map_err(|e| Error(format!(".tempa/mcp.toml tidak valid: {e}")))?
                .servers
        }
        Err(_) => BTreeMap::new(),
    };

    let mut claude_extra = None;
    let mut bases = BTreeMap::new();
    for (rel, path) in walk_source(root, &t.join("targets"))? {
        if rel == "claude.md" {
            claude_extra = Some(crate::md::normalize_eol(&std::fs::read_to_string(path)?));
            continue;
        }
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        let invalid = |e: String| Error(format!("{TEMPA_DIR}/targets/{rel} tidak valid: {e}"));
        if rel.ends_with(".json") {
            serde_json::from_str::<serde_json::Value>(&text).map_err(|e| invalid(e.to_string()))?;
        } else if rel.ends_with(".toml") {
            text.parse::<toml_edit::DocumentMut>()
                .map_err(|e| invalid(e.to_string()))?;
        }
        bases.insert(rel, bytes);
    }

    Ok(Source {
        agents_md,
        commands: items(root, &t.join("commands"))?,
        agents: items(root, &t.join("agents"))?,
        skills: skills.into_values().collect(),
        mcp,
        claude_extra,
        bases,
    })
}
