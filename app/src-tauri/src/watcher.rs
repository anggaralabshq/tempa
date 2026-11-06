//! Watch a project: source edits sync automatically (no-clobber, like `tempa sync`);
//! direct edits to generated files are reported, never absorbed on their own.

use notify::{RecursiveMode, Watcher};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Duration;
use tempa_core::config::Config;
use tempa_core::sync;

const QUIET: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Contract source: an edit re-syncs the targets.
    Source,
    /// Anything else in the repo: may change drift (generated files, lock).
    Other,
    /// Never worth a look.
    Ignore,
}

pub fn classify(rel: &str, context: &str) -> Kind {
    if rel
        .split('/')
        .any(|c| matches!(c, ".git" | "target" | "node_modules"))
        || rel.starts_with(".tempa/hooks/")
        || rel.ends_with(".tempa-tmp")
    // write_atomic's temp file
    {
        Kind::Ignore
    } else if rel == context
        || rel == "tempa.toml"
        || (rel.starts_with(".tempa/") && rel != sync::LOCK)
    {
        Kind::Source
    } else {
        Kind::Other
    }
}

/// `first` plus everything that arrives until the channel stays quiet for `quiet`.
pub fn burst<T>(rx: &Receiver<T>, first: T, quiet: Duration) -> Vec<T> {
    let mut all = vec![first];
    loop {
        match rx.recv_timeout(quiet) {
            Ok(x) => all.push(x),
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => return all,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Update {
    pub root: String,
    /// Files the automatic sync wrote (lock included).
    pub synced: Vec<String>,
    /// Targets the sync refused to overwrite.
    pub conflicts: Vec<String>,
    pub drift: usize,
    pub error: Option<String>,
}

/// Dropping it stops the watcher and its thread.
pub struct Watch(#[allow(dead_code)] notify::RecommendedWatcher);

pub fn watch(root: &Path, on: impl Fn(Update) + Send + 'static) -> Result<Watch, String> {
    // FSEvents reports canonical paths (/private/var/... on macOS).
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let (tx, rx) = mpsc::channel::<Vec<PathBuf>>();
    let mut w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(e) = res {
            let _ = tx.send(e.paths);
        }
    })
    .map_err(|e| e.to_string())?;
    w.watch(&root, RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let mut last = drift(&root).unwrap_or(0);
        while let Ok(first) = rx.recv() {
            let context = Config::load(&root)
                .map(|c| c.docs.context)
                .unwrap_or_else(|_| "AGENTS.md".into());
            let kinds: Vec<Kind> = burst(&rx, first, QUIET)
                .into_iter()
                .flatten()
                .filter_map(|p| {
                    let rel = p
                        .strip_prefix(&root)
                        .ok()?
                        .to_string_lossy()
                        .replace('\\', "/");
                    Some(classify(&rel, &context))
                })
                .collect();
            let mut u = Update {
                root: root.display().to_string(),
                ..Update::default()
            };
            if kinds.contains(&Kind::Source) {
                if root.join("tempa.toml").exists() {
                    match sync::sync_with(&root, &|_| false) {
                        Ok((synced, conflicts)) => (u.synced, u.conflicts) = (synced, conflicts),
                        Err(e) => u.error = Some(e.to_string()),
                    }
                }
            } else if !kinds.contains(&Kind::Other) {
                continue;
            }
            match drift(&root) {
                Ok(n) => u.drift = n,
                Err(e) => u.error = u.error.or(Some(e)),
            }
            // tempa's own writes leave drift unchanged, so they never come back as an alert.
            if kinds.contains(&Kind::Source) || u.drift != last || u.error.is_some() {
                last = u.drift;
                on(u);
            }
        }
    });
    Ok(Watch(w))
}

fn drift(root: &Path) -> Result<usize, String> {
    if !root.join("tempa.toml").exists() {
        return Ok(0);
    }
    sync::drift(root)
        .map(|d| d.len())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn classify_table() {
        let ctx = "AGENTS.md";
        for (path, want) in [
            ("AGENTS.md", Kind::Source),
            ("tempa.toml", Kind::Source),
            (".tempa/commands/review.md", Kind::Source),
            (".tempa/mcp.toml", Kind::Source),
            (".tempa/lock.json", Kind::Other),
            (".tempa/hooks/pre-commit", Kind::Ignore),
            (".tempa/lock.json.tempa-tmp", Kind::Ignore),
            (".git/index", Kind::Ignore),
            ("target/debug/x", Kind::Ignore),
            ("app/node_modules/x/y.js", Kind::Ignore),
            ("CLAUDE.md", Kind::Other),
            ("src/main.rs", Kind::Other),
        ] {
            assert_eq!(classify(path, ctx), want, "{path}");
        }
    }

    #[test]
    fn debounce_collapses_burst() {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for i in 0..10 {
                tx.send(i).unwrap();
                std::thread::sleep(Duration::from_millis(5));
            }
            std::thread::sleep(Duration::from_millis(600));
            tx.send(99).unwrap();
        });
        let first = rx.recv().unwrap();
        assert_eq!(
            burst(&rx, first, Duration::from_millis(200)),
            (0..10).collect::<Vec<_>>()
        );
        assert_eq!(rx.recv().unwrap(), 99);
    }

    fn git(dir: &std::path::Path, args: &[&str]) {
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(dir)
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    }

    fn adopted() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        git(d.path(), &["init", "-q", "-b", "main"]);
        tempa_core::init::init(d.path()).unwrap();
        std::fs::create_dir_all(d.path().join(".tempa/commands")).unwrap();
        std::fs::write(
            d.path().join(".tempa/commands/review.md"),
            "---\ndescription: R\n---\nv1\n",
        )
        .unwrap();
        tempa_core::sync::sync(d.path()).unwrap();
        d
    }

    fn next(rx: &mpsc::Receiver<Update>) -> Update {
        rx.recv_timeout(Duration::from_secs(10))
            .expect("watcher update")
    }

    #[test]
    fn source_edit_syncs_target_and_own_writes_do_not_alert() {
        let d = adopted();
        let (tx, rx) = mpsc::channel();
        let _w = watch(d.path(), move |u| {
            let _ = tx.send(u);
        })
        .unwrap();
        std::thread::sleep(Duration::from_millis(300));
        std::fs::write(
            d.path().join(".tempa/commands/review.md"),
            "---\ndescription: R\n---\nv2\n",
        )
        .unwrap();
        let u = next(&rx);
        assert!(
            u.synced.iter().any(|p| p == ".claude/commands/review.md"),
            "{u:?}"
        );
        assert_eq!(u.drift, 0, "{u:?}");
        let claude = std::fs::read_to_string(d.path().join(".claude/commands/review.md")).unwrap();
        assert!(claude.contains("v2"));
        // tempa's own writes must not come back as an alert.
        if let Ok(u) = rx.recv_timeout(Duration::from_secs(2)) {
            panic!("tempa's own sync came back as an update: {u:?}");
        }
    }

    #[test]
    fn target_edit_reports_drift_without_touching_it() {
        let d = adopted();
        let (tx, rx) = mpsc::channel();
        let _w = watch(d.path(), move |u| {
            let _ = tx.send(u);
        })
        .unwrap();
        std::thread::sleep(Duration::from_millis(300));
        let claude = d.path().join("CLAUDE.md");
        let edited = std::fs::read_to_string(&claude).unwrap() + "hand edit\n";
        std::fs::write(&claude, &edited).unwrap();
        let u = next(&rx);
        assert_eq!(u.drift, 1, "{u:?}");
        assert!(u.synced.is_empty());
        assert_eq!(std::fs::read_to_string(&claude).unwrap(), edited);
    }
}
