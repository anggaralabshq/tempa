//! What the host decides alone on an agent's permission request, by lifecycle phase
//! (spec: M3 "Tool policy"). Never answers `*_always`: every call is decided again.

use serde_json::Value;
use std::path::{Component, Path, PathBuf};
use tempa_core::config::Config;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    /// No task: chat in the project root.
    Explore,
    Plan,
    /// approved, implement, verify, record.
    Approved,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Allow,
    Deny(String),
    Ask,
}

/// Phase of a task chat from its plan `state` (unreadable = still planning).
pub fn phase(state: Option<&str>) -> Phase {
    match state {
        Some("approved" | "implement" | "verify" | "record") => Phase::Approved,
        _ => Phase::Plan,
    }
}

/// Case-insensitive file systems (Windows, default macOS) compare folded.
const FOLD: bool = cfg!(any(windows, target_os = "macos"));

fn fold(s: &str) -> String {
    let s = s.replace('\\', "/");
    if FOLD { s.to_lowercase() } else { s }
}

/// Canonical form: the deepest existing ancestor canonicalized by the OS (symlinks
/// and `..` resolved in that order), the missing tail appended. None when the tail
/// holds `..`: it cannot be resolved without the files existing.
fn real(p: &Path) -> Option<PathBuf> {
    let parts: Vec<Component> = p.components().collect();
    for i in (1..=parts.len()).rev() {
        let head: PathBuf = parts[..i].iter().collect();
        let Ok(mut h) = head.canonicalize() else {
            continue;
        };
        for c in &parts[i..] {
            match c {
                Component::Normal(n) => h.push(n),
                Component::CurDir => {}
                _ => return None,
            }
        }
        return Some(h);
    }
    None
}

/// `path` relative to `scope` (`/`-separated, folded), or None when outside it.
fn inside(scope: &Path, path: &str) -> Option<String> {
    let root = fold(&real(scope)?.to_string_lossy());
    let full = fold(&real(&scope.join(path))?.to_string_lossy());
    let root = root.trim_end_matches('/');
    if full == root {
        return Some(String::new());
    }
    full.strip_prefix(&format!("{root}/")).map(String::from)
}

/// The spec's ordered rules. Every path of the call must pass.
pub fn decide(phase: Phase, kind: &str, paths: &[String], scope: &Path, cfg: &Config) -> Verdict {
    let mut folded = cfg.clone();
    folded.gates.protected_paths = cfg.gates.protected_paths.iter().map(|g| fold(g)).collect();
    for p in paths {
        let Some(rel) = inside(scope, p) else {
            return Verdict::Deny(format!("{p} is outside {}", scope.display()));
        };
        if tempa_core::check::is_protected(&folded, &rel) {
            return Verdict::Deny(format!("{rel} is a protected path"));
        }
    }
    match kind {
        "edit" | "delete" | "move" if phase != Phase::Approved => {
            Verdict::Deny("edits wait for an approved plan".into())
        }
        "edit" | "delete" | "move" if paths.is_empty() => Verdict::Ask,
        "edit" | "delete" | "move" | "read" | "search" | "fetch" | "think" => Verdict::Allow,
        _ => Verdict::Ask,
    }
}

/// The agent's option for `v`: `allow_once` / `reject_once`, never `*_always`.
/// None = ask the user.
pub fn pick(options: &[Value], v: &Verdict) -> Option<String> {
    let kind = match v {
        Verdict::Allow => "allow_once",
        Verdict::Deny(_) => "reject_once",
        Verdict::Ask => return None,
    };
    options
        .iter()
        .find(|o| o["kind"] == kind)
        .and_then(|o| o["optionId"].as_str().map(String::from))
}

const TOOLS: &str = "tempa_context, tempa_status, tempa_plan, tempa_decide, tempa_log_change, tempa_check, tempa_pr_body";

/// Prepended to a session's first prompt: where the agent is and what the host enforces.
pub fn preamble(task: Option<(&str, &str)>) -> String {
    let Some((id, state)) = task else {
        return format!(
            "[tempa] Explore mode: no task, project root. Reads and searches are allowed; edits are rejected; commands ask the user. To change code, the user starts a task. tempa MCP tools: {TOOLS}.\n\n"
        );
    };
    format!(
        "[tempa] Task {id} (state: {state}), in its own worktree. The host enforces:\n\
         1. Protected paths and files outside this folder are rejected.\n\
         2. Until the plan is approved, edits, deletes and moves are rejected.\n\
         3. Reads and searches are allowed.\n\
         4. Commands and other tools ask the user.\n\
         5. After approval, edits in this folder are allowed one call at a time.\n\
         Write the plan with tempa_plan, verify with tempa_check {{\"harness\": true}}, record with tempa_log_change. tempa MCP tools: {TOOLS}.\n\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cfg() -> Config {
        let mut c = Config::parse(&tempa_core::config::default_toml("x")).unwrap();
        c.gates.protected_paths = vec![
            ".env*".into(),
            ".github/workflows/**".into(),
            "tempa.toml".into(),
        ];
        c
    }

    fn d(phase: Phase, kind: &str, paths: &[&str], scope: &Path) -> Verdict {
        let paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
        decide(phase, kind, &paths, scope, &cfg())
    }

    fn denied(v: Verdict) -> bool {
        matches!(v, Verdict::Deny(_))
    }

    #[test]
    fn policy_table() {
        let t = tempfile::tempdir().unwrap();
        let s = t.path();
        assert!(denied(d(Phase::Plan, "edit", &["src/a.rs"], s)));
        assert_eq!(d(Phase::Plan, "read", &["src/a.rs"], s), Verdict::Allow);
        assert_eq!(d(Phase::Plan, "execute", &[], s), Verdict::Ask);
        assert_eq!(d(Phase::Approved, "edit", &["src/a.rs"], s), Verdict::Allow);
        assert_eq!(
            d(Phase::Approved, "delete", &["src/new/b.rs"], s),
            Verdict::Allow
        );
        assert!(denied(d(Phase::Approved, "edit", &[".env.local"], s)));
        assert!(denied(d(
            Phase::Approved,
            "edit",
            &[".github/workflows/ci.yml"],
            s
        )));
        assert!(denied(d(Phase::Approved, "read", &[".env"], s)));
        assert!(denied(d(Phase::Explore, "edit", &["src/a.rs"], s)));
        assert_eq!(d(Phase::Approved, "edit", &[], s), Verdict::Ask);
        assert_eq!(d(Phase::Approved, "execute", &[], s), Verdict::Ask);
        assert_eq!(d(Phase::Explore, "search", &[], s), Verdict::Allow);
        assert_eq!(phase(None), Phase::Plan);
        assert_eq!(phase(Some("plan")), Phase::Plan);
        assert_eq!(phase(Some("implement")), Phase::Approved);
    }

    #[test]
    fn paths_outside_scope_are_denied() {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("wt");
        std::fs::create_dir_all(s.join("src")).unwrap();
        let other = t.path().join("other");
        std::fs::create_dir_all(&other).unwrap();
        let abs = other.join("x").to_string_lossy().to_string();
        assert!(denied(d(Phase::Approved, "edit", &["../x"], &s)));
        assert!(denied(d(Phase::Approved, "edit", &["src/../../x"], &s)));
        assert!(denied(d(Phase::Approved, "edit", &[&abs], &s)));
        assert!(denied(d(Phase::Approved, "read", &[&abs], &s)));
        // An absolute path inside the scope is fine.
        let inside = s.join("src/a.rs").to_string_lossy().to_string();
        assert_eq!(d(Phase::Approved, "edit", &[&inside], &s), Verdict::Allow);
        // One bad path in a multi-path call denies the call.
        assert!(denied(d(
            Phase::Approved,
            "move",
            &["src/a.rs", "../x"],
            &s
        )));
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&other, s.join("out")).unwrap();
            assert!(denied(d(Phase::Approved, "edit", &["out/x"], &s)));
            // `..` after a symlink is resolved by the OS, not lexically.
            std::fs::create_dir_all(other.join("deep")).unwrap();
            std::os::unix::fs::symlink(other.join("deep"), s.join("src/deep")).unwrap();
            assert!(denied(d(
                Phase::Approved,
                "edit",
                &["src/deep/../escape.txt"],
                &s
            )));
            assert!(denied(d(
                Phase::Approved,
                "edit",
                &["src/deep/../../wt/../x"],
                &s
            )));
            // A protected file reached through a symlinked dir inside the scope.
            std::os::unix::fs::symlink(&s, s.join("self")).unwrap();
            assert!(denied(d(Phase::Approved, "edit", &["self/tempa.toml"], &s)));
        }
        #[cfg(windows)]
        {
            let upper = s.join("SRC").join("A.RS").to_string_lossy().to_uppercase();
            assert_eq!(d(Phase::Approved, "edit", &[&upper], &s), Verdict::Allow);
            assert!(denied(d(Phase::Approved, "edit", &["TEMPA.TOML"], &s)));
        }
    }

    #[test]
    fn pick_never_allows_always() {
        let opts = [
            json!({"optionId": "a2", "kind": "allow_always"}),
            json!({"optionId": "a1", "kind": "allow_once"}),
            json!({"optionId": "r2", "kind": "reject_always"}),
            json!({"optionId": "r1", "kind": "reject_once"}),
        ];
        assert_eq!(pick(&opts, &Verdict::Allow).as_deref(), Some("a1"));
        assert_eq!(
            pick(&opts, &Verdict::Deny("no".into())).as_deref(),
            Some("r1")
        );
        assert_eq!(pick(&opts, &Verdict::Ask), None);
        assert_eq!(pick(&opts[..1], &Verdict::Allow), None);
    }

    #[test]
    fn preamble_names_task_and_tools() {
        let p = preamble(Some(("20261106-x", "plan")));
        assert!(p.contains("20261106-x") && p.contains("plan"));
        for t in [
            "tempa_context",
            "tempa_status",
            "tempa_plan",
            "tempa_decide",
            "tempa_log_change",
            "tempa_check",
            "tempa_pr_body",
        ] {
            assert!(p.contains(t), "{t}");
        }
        assert!(preamble(None).contains("Explore"));
    }
}
