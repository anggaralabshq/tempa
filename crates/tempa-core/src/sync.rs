//! Write rendered targets, keep `.tempa/lock.json`, detect drift.

use crate::config::Config;
use crate::targets::{OutFile, managed_dirs, render_all};
use crate::{Error, Finding, Result, gitx, source, write_atomic};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const LOCK: &str = ".tempa/lock.json";

/// SHA-256 hex of `bytes` with CRLF normalized to LF (Windows checkouts are not drift).
pub fn hash(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
            i += 1;
            continue;
        }
        h.update([bytes[i]]);
        i += 1;
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lock {
    pub files: BTreeMap<String, String>,
}

impl Lock {
    /// Missing lock = empty; unreadable lock (e.g. merge-conflict markers) = error.
    pub fn load(root: &Path) -> Result<Lock> {
        match std::fs::read(root.join(LOCK)) {
            Ok(b) => serde_json::from_slice(&b).map_err(|e| {
                Error(format!(
                    "{LOCK} tidak valid ({e}); selesaikan konfliknya, atau hapus file itu lalu jalankan tempa sync --force"
                ))
            }),
            Err(_) => Ok(Lock::default()),
        }
    }

    fn save(&self, root: &Path) -> Result<()> {
        let text = serde_json::to_string_pretty(self)? + "\n";
        write_atomic(&root.join(LOCK), text.as_bytes())
    }
}

fn rendered(root: &Path) -> Result<(Config, Vec<OutFile>)> {
    let cfg = Config::load(root)?;
    let src = source::load(root, &cfg)?;
    let files = render_all(&cfg, &src).files;
    Ok((cfg, files))
}

fn disk_hash(root: &Path, rel: &str) -> Option<String> {
    std::fs::read(root.join(rel)).ok().map(|b| hash(&b))
}

/// Render and write changed targets; remove outputs no longer rendered (only if
/// untouched since the last sync). A target that exists but is neither the current
/// render nor what tempa last wrote is a conflict and is left alone unless `allow(path)`.
/// Returns (changed or removed paths, including the lock; conflicts).
pub fn sync_with(root: &Path, allow: &dyn Fn(&str) -> bool) -> Result<(Vec<String>, Vec<String>)> {
    let (_, files) = rendered(root)?;
    let old = Lock::load(root)?;
    let mut lock = Lock::default();
    let mut changed = Vec::new();
    let mut conflicts = Vec::new();
    for f in &files {
        let h = hash(&f.bytes);
        match disk_hash(root, &f.path) {
            Some(d) if d == h => {}
            Some(d) if old.files.get(&f.path) != Some(&d) && !allow(&f.path) => {
                conflicts.push(f.path.clone());
                if let Some(prev) = old.files.get(&f.path) {
                    lock.files.insert(f.path.clone(), prev.clone());
                }
                continue;
            }
            _ => {
                write_atomic(&root.join(&f.path), &f.bytes)?;
                changed.push(f.path.clone());
            }
        }
        lock.files.insert(f.path.clone(), h);
    }
    for (path, h) in &old.files {
        if !lock.files.contains_key(path) && disk_hash(root, path).as_deref() == Some(h.as_str()) {
            std::fs::remove_file(root.join(path))?;
            changed.push(path.clone());
        }
    }
    if lock != old || !root.join(LOCK).exists() {
        lock.save(root)?;
        changed.push(LOCK.to_string());
    }
    changed.sort();
    Ok((changed, conflicts))
}

/// Safe sync: refuses (after writing everything else) when a target would clobber
/// a file tempa does not own or one that was edited directly.
pub fn sync(root: &Path) -> Result<Vec<String>> {
    let (changed, conflicts) = sync_with(root, &|_| false)?;
    if !conflicts.is_empty() {
        return Err(Error(format!(
            "tidak ditimpa karena bukan hasil tempa atau sudah diedit langsung: {} — serap dengan `tempa import --yes`, atau timpa dengan `tempa sync --force`",
            conflicts.join(", ")
        )));
    }
    Ok(changed)
}

/// Overwrite every target with the current render.
pub fn sync_force(root: &Path) -> Result<Vec<String>> {
    Ok(sync_with(root, &|_| true)?.0)
}

/// Target dirs `tempa import` can absorb new files from: (dir, `.tempa/<kind>`).
pub const ITEM_DIRS: [(&str, &str); 8] = [
    (".claude/commands", "commands"),
    (".claude/agents", "agents"),
    (".claude/skills", "skills"),
    (".agents/skills", "skills"),
    (".opencode/commands", "commands"),
    (".opencode/agents", "agents"),
    (".cursor/commands", "commands"),
    (".gemini/agents", "agents"),
];

/// Shared config targets whose non-tempa keys `tempa import` keeps as a base.
pub const CONFIG_FILES: [&str; 6] = [
    ".claude/settings.json",
    ".mcp.json",
    ".cursor/mcp.json",
    ".gemini/settings.json",
    "opencode.json",
    ".codex/config.toml",
];

/// Direct edits to this target can be absorbed by `tempa import`.
pub fn absorbs_edit(path: &str) -> bool {
    path == "CLAUDE.md"
        || path == ".cursor/rules/tempa.mdc"
        || path.starts_with(".claude/")
        || CONFIG_FILES.contains(&path)
}

/// A new file at this path can be absorbed by `tempa import`.
pub fn absorbs_new(path: &str) -> bool {
    ITEM_DIRS
        .iter()
        .any(|(dir, _)| path.starts_with(&format!("{dir}/")))
}

/// OS litter and git-ignored files are never drift and never imported.
pub fn junk(root: &Path, rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    matches!(name, ".DS_Store" | "Thumbs.db" | "desktop.ini")
        || rel.ends_with(".tempa-tmp")
        || gitx::is_ignored(root, rel)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Drift {
    /// Target edited directly (absorb with `tempa import`).
    Modified(String),
    /// Target file deleted (restore with `tempa sync`).
    Missing(String),
    /// File inside a tempa-owned dir that no source produces (absorb with `tempa import`).
    Untracked(String),
    /// Source changed, target not re-synced yet (`tempa sync`).
    Stale(String),
}

pub fn drift(root: &Path) -> Result<Vec<Drift>> {
    let (cfg, files) = rendered(root)?;
    let lock = Lock::load(root)?;
    let mut out = Vec::new();
    let mut expected = BTreeSet::new();
    for f in &files {
        expected.insert(f.path.clone());
        let want = hash(&f.bytes);
        match disk_hash(root, &f.path) {
            None => out.push(Drift::Missing(f.path.clone())),
            Some(h) if h == want => {}
            Some(h) if lock.files.get(&f.path) == Some(&h) => {
                out.push(Drift::Stale(f.path.clone()))
            }
            Some(_) => out.push(Drift::Modified(f.path.clone())),
        }
    }
    let dirs: BTreeSet<&str> = cfg
        .agents
        .enabled
        .iter()
        .flat_map(|a| managed_dirs(*a).iter().copied())
        .collect();
    for dir in dirs {
        for (rel, _) in source::walk(&root.join(dir))? {
            let path = format!("{dir}/{rel}");
            if expected.contains(&path) || junk(root, &path) {
                continue;
            }
            // An unchanged output whose source was removed: `tempa sync` deletes it.
            if lock
                .files
                .get(&path)
                .is_some_and(|h| disk_hash(root, &path).as_ref() == Some(h))
            {
                out.push(Drift::Stale(path));
            } else {
                out.push(Drift::Untracked(path));
            }
        }
    }
    Ok(out)
}

pub fn drift_findings(d: &[Drift]) -> Vec<Finding> {
    d.iter()
        .map(|x| match x {
            Drift::Modified(p) if absorbs_edit(p) => Finding::error(
                "drift.modified",
                Some(p.clone()),
                "diedit langsung — jalankan tempa import --yes untuk menyerap",
            ),
            Drift::Modified(p) => Finding::error(
                "drift.modified",
                Some(p.clone()),
                "diedit langsung — pindahkan editan ke sumbernya (AGENTS.md atau .tempa/), lalu jalankan tempa sync --force",
            ),
            Drift::Untracked(p) if absorbs_new(p) => Finding::error(
                "drift.untracked",
                Some(p.clone()),
                "file baru di folder milik tempa — jalankan tempa import --yes untuk menyerap",
            ),
            Drift::Untracked(p) => Finding::error(
                "drift.untracked",
                Some(p.clone()),
                "file baru di folder milik tempa — buat sumbernya di .tempa/commands/ (markdown) lalu hapus file ini",
            ),
            Drift::Missing(p) => Finding::error(
                "drift.missing",
                Some(p.clone()),
                "file hasil generate hilang — jalankan tempa sync",
            ),
            Drift::Stale(p) => Finding::error(
                "drift.stale",
                Some(p.clone()),
                "sumber berubah, belum di-sync — jalankan tempa sync",
            ),
        })
        .collect()
}

/// For the pre-commit hook: sync when drift is only stale/missing and every source
/// change is staged (outputs must match what is committed); otherwise write nothing.
pub fn sync_fix(root: &Path) -> Result<(Vec<String>, Vec<Finding>)> {
    if gitx::is_repo(root) {
        let cfg = Config::load(root)?;
        let paths = [
            cfg.docs.context.as_str(),
            crate::config::FILE,
            source::TEMPA_DIR,
        ];
        let unstaged: Vec<Finding> = gitx::unstaged(root, &paths)?
            .into_iter()
            .filter(|p| p != LOCK)
            .map(|p| {
                Finding::error(
                    "sync.unstaged_source",
                    Some(p),
                    "sumber punya perubahan yang belum di-stage — git add (atau git stash) dulu agar hasil generate sama dengan yang di-commit",
                )
            })
            .collect();
        if !unstaged.is_empty() {
            return Ok((vec![], unstaged));
        }
    }
    let blockers: Vec<Drift> = drift(root)?
        .into_iter()
        .filter(|d| matches!(d, Drift::Modified(_) | Drift::Untracked(_)))
        .collect();
    if !blockers.is_empty() {
        return Ok((vec![], drift_findings(&blockers)));
    }
    Ok((sync(root)?, vec![]))
}
