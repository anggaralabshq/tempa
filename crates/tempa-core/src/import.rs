//! Brownfield import: absorb agent-specific files (mostly `.claude/`) into the
//! canonical source (`AGENTS.md` + `.tempa/` + `tempa.toml`).

use crate::config::{self, Config};
use crate::md::{normalize_eol, paragraphs, split_front};
use crate::source::{McpFile, McpServer, TEMPA_DIR, walk};
use crate::sync::{Lock, hash};
use crate::targets::HEADER_MD;
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
}

/// Everything import would change, computed before touching disk.
#[derive(Default)]
struct Plan {
    create_config: bool,
    files: BTreeMap<String, Vec<u8>>,
    report: ImportReport,
}

fn norm(p: &str) -> String {
    p.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn read(root: &Path, rel: &str) -> Option<String> {
    std::fs::read_to_string(root.join(rel)).ok().map(|s| normalize_eol(&s))
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
    if rel.starts_with('~') || p.is_absolute() || p.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Include::Outside;
    }
    let (Ok(base), Ok(full)) = (std::fs::canonicalize(root), std::fs::canonicalize(root.join(p))) else {
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
        if let Some(path) = t.strip_prefix('@').filter(|p| !p.is_empty() && !p.contains(char::is_whitespace)) {
            match read_include(root, path) {
                Include::Text(inc) => {
                    out.push_str(inc.trim_end());
                    out.push('\n');
                    continue;
                }
                Include::Missing => gaps.push(format!("{from}: @{path} tidak ditemukan, baris dibiarkan")),
                Include::Outside => gaps.push(format!("{from}: @{path} di luar repo, tidak di-inline")),
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

fn merge_context(root: &Path, cfg: &Config, plan: &mut Plan) {
    let ctx = cfg.docs.context.as_str();
    let base = read(root, ctx).unwrap_or_default();
    let mut text = base.clone();
    let mut seen: HashSet<String> = paragraphs(&base).iter().map(|p| norm(p)).collect();

    let mut sources: Vec<String> = ["CLAUDE.md", "GEMINI.md", ".cursorrules"].iter().map(|s| s.to_string()).collect();
    if let Ok(rules) = walk(&root.join(".cursor/rules")) {
        sources.extend(rules.into_iter().filter(|(r, _)| r.ends_with(".mdc")).map(|(r, _)| format!(".cursor/rules/{r}")));
    }
    sources.push(".github/copilot-instructions.md".into());

    for rel in sources.iter().filter(|r| r.as_str() != ctx) {
        let Some(raw) = read(root, rel) else { continue };
        let body = if rel.ends_with(".mdc") { split_front(&raw).1.to_string() } else { raw };
        let r = &mut plan.report;
        let expanded = expand(root, &body, ctx, &mut r.gaps, rel);
        let mut fresh = Vec::new();
        for p in paragraphs(&expanded) {
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
        if text.trim().is_empty() {
            text = expanded.trim_start_matches('\n').to_string();
        } else {
            if !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&format!("\n## Imported from {rel}\n\n{}\n", fresh.join("\n\n")));
        }
    }
    if text != base {
        plan.files.insert(ctx.to_string(), text.into_bytes());
    }
}

/// Copy `from` (a target path) into `to` (a canonical path).
/// Generated targets (in the lock) are absorbed only when edited directly.
fn copy_in(root: &Path, lock: &Lock, from: &str, to: &str, plan: &mut Plan) -> Result<()> {
    let bytes = std::fs::read(root.join(from))?;
    if let Some(locked) = lock.files.get(from) {
        if *locked == hash(&bytes) {
            return Ok(()); // untouched generated file
        }
        let text = String::from_utf8_lossy(&bytes);
        let stripped = text.replacen(&format!("{HEADER_MD}\n"), "", 1).replacen(&format!("{HEADER_MD}\r\n"), "", 1);
        plan.files.insert(to.to_string(), stripped.into_bytes());
        return Ok(());
    }
    match std::fs::read(root.join(to)) {
        Ok(existing) if hash(&existing) == hash(&bytes) => {}
        Ok(_) => plan.report.gaps.push(format!("{to} sudah ada dengan isi berbeda dari {from}; tidak ditimpa")),
        Err(_) => {
            plan.files.insert(to.to_string(), bytes);
        }
    }
    Ok(())
}

fn copy_items(root: &Path, lock: &Lock, plan: &mut Plan) -> Result<()> {
    for kind in ["commands", "agents", "skills"] {
        for (rel, _) in walk(&root.join(".claude").join(kind))? {
            copy_in(root, lock, &format!(".claude/{kind}/{rel}"), &format!("{TEMPA_DIR}/{kind}/{rel}"), plan)?;
        }
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
    GROUPS.iter().find(|(_, kws)| kws.iter().any(|k| cmd.contains(k))).map(|(n, _)| *n)
}

fn settings(root: &Path, lock: &Lock, cfg: &Config, plan: &mut Plan) -> Result<()> {
    let from = ".claude/settings.json";
    let Ok(text) = std::fs::read_to_string(root.join(from)) else { return Ok(()) };
    let v: Value =
        serde_json::from_str(&text).map_err(|e| Error(format!("{from} bukan JSON valid: {e}")))?;
    copy_in(root, lock, from, &format!("{TEMPA_DIR}/targets/{from}"), plan)?;
    let r = &mut plan.report;

    let mut protected: BTreeSet<String> = cfg.gates.protected_paths.iter().cloned().collect();
    for rule in v["permissions"]["deny"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        let path = ["Edit(", "Write(", "MultiEdit(", "Read("]
            .iter()
            .find_map(|p| rule.strip_prefix(p))
            .and_then(|r| r.strip_suffix(')'));
        match path {
            Some(p) if !p.starts_with("//") && !p.starts_with('~') => {
                let p = p.trim_start_matches("./").trim_start_matches('/').to_string();
                if protected.insert(p.clone()) {
                    r.protected_added.push(p);
                }
            }
            _ => r.gaps.push(format!("deny rule {rule} tidak punya padanan lintas-agent")),
        }
    }

    let mut harness: BTreeSet<String> = cfg.harness.iter().map(|(n, _)| n.clone()).collect();
    let hooks = v["hooks"].as_object().into_iter().flatten().flat_map(|(_, groups)| {
        groups.as_array().into_iter().flatten().flat_map(|g| g["hooks"].as_array().into_iter().flatten())
    });
    for h in hooks.filter(|h| h["type"] == "command") {
        let Some(cmd) = h["command"].as_str() else { continue };
        let name = if cmd.contains("$CLAUDE") || cmd.contains("jq") || cmd.contains('|') { None } else { classify(cmd) };
        match name {
            Some(n) if harness.insert(n.to_string()) => r.harness_added.push((n.to_string(), cmd.to_string())),
            Some(_) => {}
            None => r.gaps.push(format!("hook `{cmd}` tidak bisa dijadikan perintah harness; tetap berlaku untuk Claude saja")),
        }
    }
    Ok(())
}

fn mcp_server(v: &Value) -> McpServer {
    let strs = |x: &Value| x.as_array().into_iter().flatten().filter_map(Value::as_str).map(String::from).collect();
    McpServer {
        command: v["command"].as_str().map(String::from),
        args: strs(&v["args"]),
        env: v["env"]
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(k, x)| x.as_str().map(|s| (k.clone(), s.to_string())))
            .collect(),
        url: v["url"].as_str().or(v["httpUrl"].as_str()).map(String::from),
    }
}

fn mcp(root: &Path, plan: &mut Plan) -> Result<()> {
    let to = format!("{TEMPA_DIR}/mcp.toml");
    let mut file: McpFile = match std::fs::read_to_string(root.join(&to)) {
        Ok(t) => toml::from_str(&t).map_err(|e| Error(format!("{to} tidak valid: {e}")))?,
        Err(_) => McpFile::default(),
    };
    let mut added = false;
    for from in [".mcp.json", ".cursor/mcp.json", ".gemini/settings.json"] {
        let Ok(text) = std::fs::read_to_string(root.join(from)) else { continue };
        let v: Value = serde_json::from_str(&text).map_err(|e| Error(format!("{from} bukan JSON valid: {e}")))?;
        for (name, s) in v["mcpServers"].as_object().into_iter().flatten() {
            if !file.servers.contains_key(name) {
                file.servers.insert(name.clone(), mcp_server(s));
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
    let lock = Lock::load(root);

    merge_context(root, &cfg, &mut plan);
    copy_items(root, &lock, &mut plan)?;
    settings(root, &lock, &cfg, &mut plan)?;
    mcp(root, &mut plan)?;

    let mut report = plan.report;
    report.written = plan.files.keys().cloned().collect();
    if plan.create_config || !report.harness_added.is_empty() || !report.protected_added.is_empty() {
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
    }
    Ok(report)
}
