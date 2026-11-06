//! Brownfield import: absorb agent-specific files (mostly `.claude/`) into the
//! canonical source (`AGENTS.md` + `.tempa/` + `tempa.toml`).

use crate::config::{self, Config};
use crate::md::{normalize_eol, paragraphs, split_front};
use crate::source::{McpFile, McpServer, TEMPA_DIR, inside, walk};
use crate::sync::{self, CONFIG_FILES, ITEM_DIRS, Lock, hash, junk};
use crate::targets::{HEADER_MD, header_toml};
use crate::{Error, Result, write_atomic};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub written: Vec<String>,
    pub merged_from: Vec<String>,
    pub duplicates_skipped: usize,
    pub harness_added: Vec<(String, String)>,
    pub protected_added: Vec<String>,
    pub gaps: Vec<String>,
    /// Files created by `tempa init` because `tempa.toml` was missing (apply only).
    pub initialized: Vec<String>,
    /// Targets written by the sync that ends an applied import.
    pub synced: Vec<String>,
    /// Targets left alone because their edits could not be absorbed (apply only).
    pub conflicts: Vec<String>,
}

/// Everything import would change, computed before touching disk.
#[derive(Default)]
struct Plan {
    create_config: bool,
    files: BTreeMap<String, Vec<u8>>,
    report: ImportReport,
    lock: Lock,
    /// Current render of the source, by target path.
    rendered: BTreeMap<String, Vec<u8>>,
    /// Targets whose content this import took into the source: sync may overwrite them.
    absorbed: BTreeSet<String>,
}

/// Where a target stands relative to tempa's own output.
enum State {
    /// Missing, equal to the current render, or untouched since tempa wrote it.
    Ours,
    /// tempa wrote it and someone edited it since.
    Edited,
    /// Exists, never written by tempa.
    Foreign,
}

impl Plan {
    fn state(&self, rel: &str, bytes: &[u8]) -> State {
        let h = hash(bytes);
        if self.rendered.get(rel).is_some_and(|r| hash(r) == h) {
            return State::Ours;
        }
        match self.lock.files.get(rel) {
            Some(l) if *l == h => State::Ours,
            Some(_) => State::Edited,
            None => State::Foreign,
        }
    }

    /// Stage `bytes` for source path `to`. An edited target replaces its source only
    /// when the source still renders to what tempa last wrote; otherwise both sides
    /// changed (e.g. uncommitted source work, or a stale lock) and nothing is overwritten.
    fn put(&mut self, root: &Path, from: &str, to: &str, bytes: Vec<u8>) {
        let current = std::fs::read(root.join(to)).ok();
        if current.as_deref().is_some_and(|c| hash(c) == hash(&bytes)) {
            self.absorbed.insert(from.to_string());
            return;
        }
        let source_moved = matches!(self.lock.files.get(from), Some(l)
            if self.rendered.get(from).map(|r| hash(r)).as_ref() != Some(l));
        if current.is_some() && source_moved {
            self.report.gaps.push(format!(
                "{to} berubah sejak sync terakhir dan {from} juga diedit; editan tidak diserap, gabungkan manual"
            ));
            return;
        }
        self.files.insert(to.to_string(), bytes);
        self.absorbed.insert(from.to_string());
    }
}

/// Drop tempa's generated-file header line, if present.
fn strip_header(text: &str) -> String {
    let toml_header = header_toml();
    let mut lines = text.split_inclusive('\n');
    let first = text.split_inclusive('\n').next().unwrap_or("");
    let t = first.trim_end_matches(['\r', '\n']);
    if t == HEADER_MD || t == toml_header {
        lines.next();
        return lines.collect();
    }
    // Header sits right after frontmatter in generated markdown.
    text.replacen(&format!("{HEADER_MD}\n"), "", 1)
        .replacen(&format!("{HEADER_MD}\r\n"), "", 1)
}

fn norm(p: &str) -> String {
    p.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn read(root: &Path, rel: &str) -> Option<String> {
    std::fs::read_to_string(root.join(rel))
        .ok()
        .map(|s| normalize_eol(&s))
}

enum Include {
    Text(String),
    Missing,
    Outside,
}

/// Read an `@include` only if it resolves (symlinks included) inside the repo.
/// Personal files like `@~/.claude/notes.md` must never be copied into AGENTS.md.
fn read_include(root: &Path, rel: &str) -> Include {
    let p = Path::new(rel);
    if rel.starts_with('~')
        || p.is_absolute()
        || p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Include::Outside;
    }
    let (Ok(base), Ok(full)) = (
        std::fs::canonicalize(root),
        std::fs::canonicalize(root.join(p)),
    ) else {
        return Include::Missing;
    };
    if !full.starts_with(&base) {
        return Include::Outside;
    }
    match std::fs::read_to_string(full) {
        Ok(s) => Include::Text(normalize_eol(&s)),
        Err(_) => Include::Missing,
    }
}

/// Drop tempa's own header and `@<context>` lines; inline other `@path` lines.
fn expand(root: &Path, text: &str, context: &str, gaps: &mut Vec<String>, from: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let t = line.trim();
        if t == HEADER_MD || t == format!("@{context}") {
            continue;
        }
        if let Some(path) = t
            .strip_prefix('@')
            .filter(|p| !p.is_empty() && !p.contains(char::is_whitespace))
        {
            match read_include(root, path) {
                Include::Text(inc) => {
                    out.push_str(inc.trim_end());
                    out.push('\n');
                    continue;
                }
                Include::Missing => {
                    gaps.push(format!("{from}: @{path} tidak ditemukan, baris dibiarkan"))
                }
                Include::Outside => {
                    gaps.push(format!("{from}: @{path} di luar repo, tidak di-inline"))
                }
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Context files merged into AGENTS.md. Generated ones (CLAUDE.md, tempa.mdc) are
/// absorbed when foreign (first adoption) or edited, minus what tempa itself wrote;
/// legacy ones (GEMINI.md, .cursorrules, ...) only on the first import, so a
/// re-import never resurrects text the team has since changed.
fn merge_context(root: &Path, cfg: &Config, plan: &mut Plan) {
    let ctx = cfg.docs.context.as_str();
    let base = read(root, ctx).unwrap_or_default();
    let mut text = base.clone();
    let mut seen: HashSet<String> = paragraphs(&base).iter().map(|p| norm(p)).collect();

    let mut sources: Vec<String> = ["CLAUDE.md", "GEMINI.md", ".cursorrules"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Ok(rules) = walk(&root.join(".cursor/rules")) {
        sources.extend(
            rules
                .into_iter()
                .filter(|(r, _)| r.ends_with(".mdc"))
                .map(|(r, _)| format!(".cursor/rules/{r}")),
        );
    }
    sources.push(".github/copilot-instructions.md".into());

    for rel in sources.iter().filter(|r| r.as_str() != ctx) {
        let Some(raw) = read(root, rel) else { continue };
        let generated = plan
            .rendered
            .get(rel)
            .map(|r| String::from_utf8_lossy(r).to_string());
        let mut own: HashSet<String> = HashSet::new();
        match &generated {
            Some(render) => match plan.state(rel, raw.as_bytes()) {
                State::Ours => continue,
                State::Edited | State::Foreign => {
                    let body = if rel.ends_with(".mdc") {
                        split_front(render).1.to_string()
                    } else {
                        render.clone()
                    };
                    own = paragraphs(&body).iter().map(|p| norm(p)).collect();
                    plan.absorbed.insert(rel.clone());
                }
            },
            None if !plan.create_config => continue,
            None => {}
        }
        let body = if rel.ends_with(".mdc") {
            split_front(&raw).1.to_string()
        } else {
            raw
        };
        let r = &mut plan.report;
        let expanded = expand(root, &body, ctx, &mut r.gaps, rel);
        let mut fresh = Vec::new();
        for p in paragraphs(&expanded) {
            if own.contains(&norm(&p)) {
                continue;
            }
            if seen.insert(norm(&p)) {
                fresh.push(p);
            } else {
                r.duplicates_skipped += 1;
            }
        }
        if fresh.is_empty() {
            continue;
        }
        r.merged_from.push(rel.clone());
        if text.trim().is_empty() && own.is_empty() {
            text = expanded.trim_start_matches('\n').to_string();
        } else if text.trim().is_empty() {
            text = fresh.join("\n\n") + "\n";
        } else {
            if !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&format!(
                "\n## Imported from {rel}\n\n{}\n",
                fresh.join("\n\n")
            ));
        }
    }
    if text != base {
        plan.files.insert(ctx.to_string(), text.into_bytes());
    }
}

/// Absorb target `from` into source `to`: foreign files are adopted, edited
/// generated files replace their source, our own output is left alone.
fn copy_in(root: &Path, from: &str, to: &str, plan: &mut Plan) -> Result<()> {
    let bytes = std::fs::read(root.join(from))?;
    match plan.state(from, &bytes) {
        State::Ours => {}
        State::Edited => {
            let text = strip_header(&String::from_utf8_lossy(&bytes));
            plan.put(root, from, to, text.into_bytes());
        }
        State::Foreign => match std::fs::read(root.join(to)) {
            Ok(existing) if hash(&existing) != hash(&bytes) => plan.report.gaps.push(format!(
                "{to} sudah ada dengan isi berbeda dari {from}; tidak ditimpa"
            )),
            _ => plan.put(root, from, to, bytes),
        },
    }
    Ok(())
}

/// `.claude/*` is absorbed fully (new and edited); other agents' dirs only for new
/// files, since their generated form drops fields the source keeps.
fn copy_items(root: &Path, plan: &mut Plan) -> Result<()> {
    for (dir, kind) in ITEM_DIRS {
        for (rel, path) in walk(&root.join(dir))? {
            let from = format!("{dir}/{rel}");
            if junk(root, &from) {
                continue;
            }
            if !inside(root, &path) {
                plan.report
                    .gaps
                    .push(format!("{from}: symlink ke luar repo, tidak disalin"));
                continue;
            }
            let known = plan.lock.files.contains_key(&from) || plan.rendered.contains_key(&from);
            if dir.starts_with(".claude/") || !known {
                copy_in(root, &from, &format!("{TEMPA_DIR}/{kind}/{rel}"), plan)?;
            }
        }
    }
    Ok(())
}

/// The MCP key each shared config file carries (owned by `.tempa/mcp.toml`).
fn mcp_key(path: &str) -> Option<&'static str> {
    match path {
        ".mcp.json" | ".cursor/mcp.json" | ".gemini/settings.json" => Some("mcpServers"),
        "opencode.json" => Some("mcp"),
        ".codex/config.toml" => Some("mcp_servers"),
        _ => None,
    }
}

/// Keep a shared config's non-tempa keys as `.tempa/targets/<path>` so sync merges
/// into them instead of replacing the file.
fn config_bases(root: &Path, plan: &mut Plan) -> Result<()> {
    for from in CONFIG_FILES {
        let Ok(bytes) = std::fs::read(root.join(from)) else {
            continue;
        };
        if matches!(plan.state(from, &bytes), State::Ours) {
            continue;
        }
        let text = strip_header(&String::from_utf8_lossy(&bytes));
        let kind = if from.ends_with(".toml") {
            "TOML"
        } else {
            "JSON"
        };
        let invalid = |e: String| Error(format!("{from} bukan {kind} valid: {e}"));
        let base = if from.ends_with(".toml") {
            let mut doc: toml_edit::DocumentMut = text
                .parse()
                .map_err(|e: toml_edit::TomlError| invalid(e.to_string()))?;
            if let Some(k) = mcp_key(from) {
                doc.remove(k);
            }
            doc.to_string()
        } else {
            let mut v: Value = serde_json::from_str(&text).map_err(|e| invalid(e.to_string()))?;
            if let (Some(k), Some(m)) = (mcp_key(from), v.as_object_mut()) {
                m.remove(k);
            }
            serde_json::to_string_pretty(&v)? + "\n"
        };
        plan.put(
            root,
            from,
            &format!("{TEMPA_DIR}/targets/{from}"),
            base.into_bytes(),
        );
    }
    Ok(())
}

/// First matching keyword group → harness name.
fn classify(cmd: &str) -> Option<&'static str> {
    const GROUPS: [(&str, &[&str]); 4] = [
        ("lint", &["eslint", "biome", "ruff", "clippy", "lint"]),
        ("format", &["prettier", "fmt", "format"]),
        ("typecheck", &["tsc", "typecheck", "mypy"]),
        ("test", &["test", "jest", "vitest", "pytest"]),
    ];
    GROUPS
        .iter()
        .find(|(_, kws)| kws.iter().any(|k| cmd.contains(k)))
        .map(|(n, _)| *n)
}

fn settings(root: &Path, cfg: &Config, plan: &mut Plan) -> Result<()> {
    let from = ".claude/settings.json";
    let Ok(text) = std::fs::read_to_string(root.join(from)) else {
        return Ok(());
    };
    let v: Value =
        serde_json::from_str(&text).map_err(|e| Error(format!("{from} bukan JSON valid: {e}")))?;
    let r = &mut plan.report;

    let mut protected: BTreeSet<String> = cfg.gates.protected_paths.iter().cloned().collect();
    for rule in v["permissions"]["deny"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        let path = ["Edit(", "Write(", "MultiEdit(", "Read("]
            .iter()
            .find_map(|p| rule.strip_prefix(p))
            .and_then(|r| r.strip_suffix(')'));
        match path {
            Some(p) if !p.starts_with("//") && !p.starts_with('~') => {
                let p = p
                    .trim_start_matches("./")
                    .trim_start_matches('/')
                    .to_string();
                if protected.insert(p.clone()) {
                    r.protected_added.push(p);
                }
            }
            _ => r
                .gaps
                .push(format!("deny rule {rule} tidak punya padanan lintas-agent")),
        }
    }

    let mut harness: BTreeSet<String> = cfg.harness.iter().map(|(n, _)| n.clone()).collect();
    let hooks = v["hooks"]
        .as_object()
        .into_iter()
        .flatten()
        .flat_map(|(_, groups)| {
            groups
                .as_array()
                .into_iter()
                .flatten()
                .flat_map(|g| g["hooks"].as_array().into_iter().flatten())
        });
    for h in hooks.filter(|h| h["type"] == "command") {
        let Some(cmd) = h["command"].as_str() else {
            continue;
        };
        let name = if cmd.contains("$CLAUDE") || cmd.contains("jq") || cmd.contains('|') {
            None
        } else {
            classify(cmd)
        };
        match name {
            Some(n) if harness.insert(n.to_string()) => r.harness_added.push((n.to_string(), cmd.to_string())),
            Some(_) => {}
            None => r.gaps.push(format!("hook `{cmd}` tidak bisa dijadikan perintah harness; tetap berlaku untuk Claude saja")),
        }
    }
    Ok(())
}

fn str_map(v: &Value) -> BTreeMap<String, String> {
    v.as_object()
        .into_iter()
        .flatten()
        .filter_map(|(k, x)| x.as_str().map(|s| (k.clone(), s.to_string())))
        .collect()
}

/// OpenCode `{env:VAR}` → `${VAR}` (the source keeps Claude's syntax).
fn from_opencode_env(m: BTreeMap<String, String>) -> BTreeMap<String, String> {
    m.into_iter()
        .map(|(k, v)| (k, v.replace("{env:", "${")))
        .collect()
}

/// One server from `.mcp.json` / `.cursor/mcp.json` / `.gemini/settings.json` /
/// `opencode.json`. Fields with no cross-agent form are reported, never dropped silently.
fn mcp_server(from: &str, name: &str, v: &Value, gaps: &mut Vec<String>) -> McpServer {
    let strs = |x: &Value| -> Vec<String> {
        x.as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect()
    };
    let mut s = McpServer::default();
    if from == "opencode.json" {
        let mut cmd = strs(&v["command"]).into_iter();
        s.command = cmd.next();
        s.args = cmd.collect();
        s.env = from_opencode_env(str_map(&v["environment"]));
        s.headers = from_opencode_env(str_map(&v["headers"]));
    } else {
        s.command = v["command"].as_str().map(String::from);
        s.args = strs(&v["args"]);
        s.env = str_map(&v["env"]);
        s.headers = str_map(&v["headers"]);
    }
    s.url = v["url"]
        .as_str()
        .or(v["httpUrl"].as_str())
        .map(String::from);
    const KNOWN: [&str; 8] = [
        "command",
        "args",
        "env",
        "environment",
        "url",
        "httpUrl",
        "headers",
        "type",
    ];
    for (k, x) in v.as_object().into_iter().flatten() {
        let ok = KNOWN.contains(&k.as_str()) && !(k == "type" && x.as_str() == Some("sse"));
        if !ok {
            gaps.push(format!(
                "mcp {name} ({from}): field {k} tidak dibawa ke agent lain"
            ));
        }
    }
    s
}

fn codex_servers(text: &str, gaps: &mut Vec<String>) -> Vec<(String, McpServer)> {
    let Ok(doc) = text.parse::<toml::Table>() else {
        return vec![];
    };
    let Some(servers) = doc.get("mcp_servers").and_then(|v| v.as_table()) else {
        return vec![];
    };
    servers
        .iter()
        .filter_map(|(name, v)| {
            let t = v.as_table()?;
            for k in t
                .keys()
                .filter(|k| !["command", "args", "env", "url"].contains(&k.as_str()))
            {
                gaps.push(format!(
                    "mcp {name} (.codex/config.toml): field {k} tidak dibawa ke agent lain"
                ));
            }
            let s: McpServer = t.clone().try_into().ok()?;
            Some((
                name.clone(),
                McpServer {
                    headers: BTreeMap::new(),
                    ..s
                },
            ))
        })
        .collect()
}

fn mcp(root: &Path, plan: &mut Plan) -> Result<()> {
    let to = format!("{TEMPA_DIR}/mcp.toml");
    let mut file: McpFile = match std::fs::read_to_string(root.join(&to)) {
        Ok(t) => toml::from_str(&t).map_err(|e| Error(format!("{to} tidak valid: {e}")))?,
        Err(_) => McpFile::default(),
    };
    let mut added = false;
    let gaps = &mut plan.report.gaps;
    for from in [
        ".mcp.json",
        ".cursor/mcp.json",
        ".gemini/settings.json",
        "opencode.json",
        ".codex/config.toml",
    ] {
        let Ok(text) = std::fs::read_to_string(root.join(from)) else {
            continue;
        };
        let found: Vec<(String, McpServer)> = if from.ends_with(".toml") {
            codex_servers(&text, gaps)
        } else {
            let v: Value = serde_json::from_str(&text)
                .map_err(|e| Error(format!("{from} bukan JSON valid: {e}")))?;
            let key = mcp_key(from).expect("json config has an mcp key");
            v[key]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(n, s)| (n.clone(), mcp_server(from, n, s, gaps)))
                .collect()
        };
        for (name, s) in found {
            if let std::collections::btree_map::Entry::Vacant(e) = file.servers.entry(name) {
                e.insert(s);
                added = true;
            }
        }
    }
    if added {
        let text = toml::to_string(&file).map_err(|e| Error(e.to_string()))?;
        plan.files.insert(to, text.into_bytes());
    }
    Ok(())
}

fn project_name(root: &Path) -> String {
    std::fs::canonicalize(root)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".into())
}

/// Import into the canonical source. `apply = false` reports without writing.
pub fn import(root: &Path, apply: bool) -> Result<ImportReport> {
    let mut plan = Plan::default();
    let cfg = match Config::load(root) {
        Ok(c) => c,
        Err(_) if !root.join(config::FILE).exists() => {
            plan.create_config = true;
            Config::parse(&config::default_toml(&project_name(root)))?
        }
        Err(e) => return Err(e),
    };
    plan.lock = Lock::load(root)?;
    let src = crate::source::load(root, &cfg)?;
    plan.rendered = crate::targets::render_all(&cfg, &src)
        .files
        .into_iter()
        .map(|f| (f.path, f.bytes))
        .collect();

    merge_context(root, &cfg, &mut plan);
    copy_items(root, &mut plan)?;
    config_bases(root, &mut plan)?;
    settings(root, &cfg, &mut plan)?;
    mcp(root, &mut plan)?;

    let mut report = plan.report;
    report.written = plan.files.keys().cloned().collect();
    if plan.create_config || !report.harness_added.is_empty() || !report.protected_added.is_empty()
    {
        report.written.push(config::FILE.to_string());
    }
    report.written.sort();

    if apply {
        if plan.create_config {
            report.initialized = crate::init::init(root)?;
        }
        for (path, bytes) in &plan.files {
            write_atomic(&root.join(path), bytes)?;
        }
        for (name, cmd) in &report.harness_added {
            config::add_harness(root, name, cmd)?;
        }
        for glob in &report.protected_added {
            config::add_protected(root, glob)?;
        }
        let absorbed = plan.absorbed;
        (report.synced, report.conflicts) = sync::sync_with(root, &|p| absorbed.contains(p))?;
    }
    Ok(report)
}
