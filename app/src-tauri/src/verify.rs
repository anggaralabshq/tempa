//! Verify run by the host (never reported by the agent): harness results land in
//! `verify.json` and move the task state.

use crate::contract::{FindingDto, dto};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tempa_core::check::{CheckOpts, check};
use tempa_core::task;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verify {
    /// RFC 3339 UTC.
    pub at: String,
    /// Every harness command passed.
    pub ok: bool,
    /// Full `tempa check` findings (contract + harness).
    pub findings: Vec<FindingDto>,
}

/// Run `tempa check` with the harness in `root`, record it in `file`, and move the
/// task: pass → record, fail → implement (only once the plan is approved).
pub fn run(root: &Path, file: &Path) -> Result<Verify, String> {
    let opts = CheckOpts {
        contract_only: false,
        base: None,
    };
    let findings = check(root, &opts).map_err(|e| e.0)?;
    let ok = !findings
        .iter()
        .any(|f| f.code.starts_with("harness.") && f.severity == tempa_core::Severity::Error);
    let v = Verify {
        at: now(),
        ok,
        findings: findings.iter().map(dto).collect(),
    };
    let text = serde_json::to_vec_pretty(&v).map_err(|e| e.to_string())?;
    tempa_core::write_atomic(file, &text).map_err(|e| e.0)?;
    if matches!(
        current_state(root).as_deref(),
        Some("approved" | "implement" | "verify")
    ) {
        task::set_state(root, if ok { "record" } else { "implement" }).map_err(|e| e.0)?;
    }
    Ok(v)
}

/// `<data>/projects/<project id>/tasks/<task id>/verify.json`.
pub fn file(data: &Path, project: &str, task: &str) -> std::path::PathBuf {
    crate::projects::data_dir(data, project)
        .join("tasks")
        .join(task)
        .join("verify.json")
}

pub fn load(file: &Path) -> Option<Verify> {
    serde_json::from_slice(&std::fs::read(file).ok()?).ok()
}

/// `state` from the current task's plan frontmatter.
pub fn current_state(root: &Path) -> Option<String> {
    let cfg = tempa_core::config::Config::load(root).ok()?;
    let id = task::current_task_id(root).ok()?;
    let text = std::fs::read_to_string(root.join(task::plan_path(&cfg, &id))).ok()?;
    text.lines()
        .take_while(|l| !l.starts_with("## "))
        .find_map(|l| l.strip_prefix("state:"))
        .map(|s| s.trim().to_string())
}

fn now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (y, m, d) = tempa_core::date::civil_from_days(secs.div_euclid(86_400));
    let t = secs.rem_euclid(86_400);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        t / 3600,
        t / 60 % 60,
        t % 60
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::path::Path;
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }

    /// Repo with tempa, one harness command `cmd`, on a fresh task branch in `state`.
    pub(crate) fn task_repo(cmd: &str, state: &str) -> (tempfile::TempDir, String) {
        let d = tempfile::tempdir().unwrap();
        git(d.path(), &["init", "-q", "-b", "main"]);
        git(d.path(), &["config", "user.name", "t"]);
        git(d.path(), &["config", "user.email", "t@x"]);
        tempa_core::init::init(d.path()).unwrap();
        tempa_core::config::add_harness(d.path(), "ok", cmd).unwrap();
        git(d.path(), &["add", "-A"]);
        git(d.path(), &["commit", "-qm", "init"]);
        let id = tempa_core::task::task_new(d.path(), "x").unwrap();
        tempa_core::task::set_state(d.path(), state).unwrap();
        (d, id)
    }

    fn state(root: &Path, id: &str) -> String {
        let plan = std::fs::read_to_string(root.join(format!("docs/plans/{id}.md"))).unwrap();
        plan.lines()
            .find_map(|l| l.strip_prefix("state: "))
            .unwrap()
            .to_string()
    }

    #[test]
    fn verify_pass_moves_to_record() {
        let (d, id) = task_repo("exit 0", "approved");
        let file = d.path().join("out/verify.json");
        let v = run(d.path(), &file).unwrap();
        assert!(v.ok, "{v:?}");
        assert_eq!(state(d.path(), &id), "record");
        assert!(load(&file).unwrap().ok);
    }

    #[test]
    fn verify_fail_moves_back_to_implement() {
        let (d, id) = task_repo("exit 1", "verify");
        let file = d.path().join("out/verify.json");
        let v = run(d.path(), &file).unwrap();
        assert!(!v.ok);
        assert!(v.findings.iter().any(|f| f.code == "harness.failed"));
        assert_eq!(state(d.path(), &id), "implement");
        assert!(!load(&file).unwrap().ok);
    }

    #[test]
    fn verify_before_approval_keeps_state_plan() {
        let (d, id) = task_repo("exit 0", "plan");
        run(d.path(), &d.path().join("v.json")).unwrap();
        assert_eq!(state(d.path(), &id), "plan");
    }
}
