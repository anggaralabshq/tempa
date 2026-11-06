//! Write rendered targets, keep `.tempa/lock.json`, detect drift.

use crate::config::Config;
use crate::targets::{OutFile, managed_dirs, render_all};
use crate::{Finding, Result, source, write_atomic};
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
    pub fn load(root: &Path) -> Lock {
        std::fs::read(root.join(LOCK))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
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
/// untouched since the last sync). Returns changed or removed paths.
pub fn sync(root: &Path) -> Result<Vec<String>> {
    let (_, files) = rendered(root)?;
    let old = Lock::load(root);
    let mut lock = Lock::default();
    let mut changed = Vec::new();
    for f in &files {
        let h = hash(&f.bytes);
        if disk_hash(root, &f.path).as_deref() != Some(h.as_str()) {
            write_atomic(&root.join(&f.path), &f.bytes)?;
            changed.push(f.path.clone());
        }
        lock.files.insert(f.path.clone(), h);
    }
    for (path, h) in &old.files {
        if !lock.files.contains_key(path) && disk_hash(root, path).as_deref() == Some(h.as_str()) {
            std::fs::remove_file(root.join(path))?;
            changed.push(path.clone());
        }
    }
    if lock != old {
        lock.save(root)?;
    }
    changed.sort();
    Ok(changed)
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
    let lock = Lock::load(root);
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
            if expected.contains(&path) || path.ends_with(".tempa-tmp") {
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
            Drift::Modified(p) => Finding::error(
                "drift.modified",
                Some(p.clone()),
                "diedit langsung — jalankan tempa import untuk menyerap",
            ),
            Drift::Untracked(p) => Finding::error(
                "drift.untracked",
                Some(p.clone()),
                "file baru di folder milik tempa — jalankan tempa import untuk menyerap",
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

/// For the pre-commit hook: sync when drift is only stale/missing; otherwise write
/// nothing and return the blocking drift.
pub fn sync_fix(root: &Path) -> Result<(Vec<String>, Vec<Drift>)> {
    let blockers: Vec<Drift> = drift(root)?
        .into_iter()
        .filter(|d| matches!(d, Drift::Modified(_) | Drift::Untracked(_)))
        .collect();
    if !blockers.is_empty() {
        return Ok((vec![], blockers));
    }
    Ok((sync(root)?, vec![]))
}
