//! Contract panel: status and actions, each the same tempa-core call as the CLI command.

use serde::Serialize;
use std::path::Path;
use tempa_core::check::{CheckOpts, check};
use tempa_core::sync::{self, Drift};
use tempa_core::{Finding, Severity, doctor, import, init};

#[derive(Debug, Serialize)]
pub struct FindingDto {
    pub code: String,
    pub severity: &'static str,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AgentDto {
    pub agent: &'static str,
    pub files: usize,
    pub gaps: usize,
    pub lifecycle: &'static str,
}

#[derive(Debug, Default, Serialize)]
pub struct ContractStatus {
    pub has_config: bool,
    /// "init" or "import" when the folder has no tempa.toml yet.
    pub offer: Option<String>,
    /// Contract and task findings (drift excluded).
    pub findings: Vec<FindingDto>,
    pub drift: Vec<FindingDto>,
    pub agents: Vec<AgentDto>,
    pub hooks_ok: bool,
    /// `tempa --version` on the terminals' PATH (the pre-commit hook runs that binary).
    pub cli_version: Option<String>,
    pub app_version: String,
    /// Set when the status itself could not be computed (e.g. invalid tempa.toml).
    pub error: Option<String>,
}

fn dto(f: &Finding) -> FindingDto {
    FindingDto {
        code: f.code.to_string(),
        severity: match f.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        },
        path: f.path.clone(),
        message: f.message.clone(),
    }
}

fn parse_cli_version(out: &str) -> Option<String> {
    let v = out.trim().strip_prefix("tempa ")?;
    (!v.is_empty()).then(|| v.to_string())
}

fn cli_version(path_env: &str) -> Option<String> {
    let out = tempa_core::command("tempa")
        .arg("--version")
        .env("PATH", path_env)
        .output()
        .ok()?;
    parse_cli_version(&String::from_utf8_lossy(&out.stdout))
}

/// Everything the panel shows. `path_env`: PATH used to find the `tempa` CLI.
pub fn status(root: &Path, path_env: Option<&str>) -> ContractStatus {
    let mut s = ContractStatus {
        app_version: env!("CARGO_PKG_VERSION").into(),
        cli_version: path_env.and_then(cli_version),
        hooks_ok: tempa_core::gitx::get_config(root, "core.hooksPath").as_deref()
            == Some(init::HOOKS_DIR),
        has_config: root.join(tempa_core::config::FILE).is_file(),
        ..Default::default()
    };
    if !root.is_dir() {
        s.error = Some(missing(root));
        return s;
    }
    if !s.has_config {
        s.offer = Some(
            if init::existing_agent_context(root).is_some() {
                "import"
            } else {
                "init"
            }
            .into(),
        );
        return s;
    }
    let opts = CheckOpts {
        contract_only: true,
        base: None,
    };
    let result = check(root, &opts).and_then(|f| Ok((f, doctor::doctor(root)?)));
    match result {
        Ok((findings, report)) => {
            let (drift, rest): (Vec<_>, Vec<_>) =
                findings.iter().partition(|f| f.code.starts_with("drift."));
            s.drift = drift.into_iter().map(dto).collect();
            s.findings = rest.into_iter().map(dto).collect();
            s.agents = report
                .rows
                .iter()
                .map(|r| AgentDto {
                    agent: r.agent.name(),
                    files: r.files,
                    gaps: r.gaps.len(),
                    lifecycle: r.lifecycle,
                })
                .collect();
            s.hooks_ok = report.hooks_ok;
        }
        Err(e) => s.error = Some(e.0),
    }
    s
}

fn missing(root: &Path) -> String {
    format!(
        "Folder not found: {}. Remove it from tempa, or move it back.",
        root.display()
    )
}

/// Files an action would change, for the confirmation dialog.
pub fn preview(root: &Path, action: &str) -> Result<Vec<String>, String> {
    match action {
        "absorb" => {
            let r = import::import(root, false).map_err(|e| e.0)?;
            let mut files = r.written;
            files.extend(r.absorbed);
            files.sort();
            files.dedup();
            Ok(files)
        }
        "discard" => Ok(sync::drift(root)
            .map_err(|e| e.0)?
            .into_iter()
            .filter_map(|d| match d {
                Drift::Modified(p) | Drift::Foreign(p) | Drift::Missing(p) | Drift::Stale(p) => {
                    Some(p)
                }
                Drift::Untracked(_) => None,
            })
            .collect()),
        other => Err(format!("unknown action {other}")),
    }
}

/// Run an action the user confirmed. Returns the changed paths.
/// `reviewed` is the list the user confirmed; Absorb, Import and Discard refuse when
/// what they would change no longer matches it.
pub fn run(root: &Path, action: &str, reviewed: &[String]) -> Result<Vec<String>, String> {
    if !root.is_dir() {
        return Err(missing(root));
    }
    let listed = match action {
        "absorb" | "import" => Some("absorb"),
        "discard" => Some("discard"),
        _ => None,
    };
    if let Some(a) = listed
        && preview(root, a)? != reviewed
    {
        return Err("The affected files changed since you reviewed them; review again.".into());
    }
    let e = |e: tempa_core::Error| e.0;
    match action {
        "absorb" | "import" => {
            let r = import::import(root, true).map_err(e)?;
            Ok(r.written
                .into_iter()
                .chain(r.initialized)
                .chain(r.synced)
                .collect())
        }
        "discard" => sync::sync_force(root).map_err(e),
        "init" => init::init(root).map_err(e),
        "hooks" => Ok(if init::install_hooks(root).map_err(e)? {
            vec![init::HOOKS_DIR.to_string()]
        } else {
            vec![]
        }),
        other => Err(format!("unknown action {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) {
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

    fn repo() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        git(d.path(), &["init", "-q", "-b", "main"]);
        git(d.path(), &["config", "user.name", "t"]);
        git(d.path(), &["config", "user.email", "t@x"]);
        d
    }

    fn adopted() -> tempfile::TempDir {
        let d = repo();
        tempa_core::init::init(d.path()).unwrap();
        tempa_core::sync::sync(d.path()).unwrap();
        d
    }

    #[test]
    fn status_of_folder_without_config_offers_import_when_claude_md_exists() {
        let d = repo();
        assert_eq!(status(d.path(), None).offer.as_deref(), Some("init"));
        std::fs::write(d.path().join("CLAUDE.md"), "# x\n").unwrap();
        let s = status(d.path(), None);
        assert!(!s.has_config);
        assert_eq!(s.offer.as_deref(), Some("import"));
        assert!(s.findings.is_empty() && s.drift.is_empty());
    }

    #[test]
    fn status_reports_drift_after_target_edit() {
        let d = adopted();
        let s = status(d.path(), None);
        assert!(
            s.has_config && s.offer.is_none() && s.drift.is_empty(),
            "{:?}",
            s.drift
        );
        assert_eq!(s.agents.len(), 6);
        assert!(s.hooks_ok);
        let claude = d.path().join("CLAUDE.md");
        std::fs::write(
            &claude,
            std::fs::read_to_string(&claude).unwrap() + "edit\n",
        )
        .unwrap();
        let s = status(d.path(), None);
        assert_eq!(s.drift.len(), 1);
        assert_eq!(s.drift[0].code, "drift.modified");
        assert_eq!(s.drift[0].path.as_deref(), Some("CLAUDE.md"));
    }

    #[test]
    fn preview_discard_lists_modified_targets() {
        let d = adopted();
        std::fs::write(d.path().join("CLAUDE.md"), "edit\n").unwrap();
        assert_eq!(preview(d.path(), "discard").unwrap(), ["CLAUDE.md"]);
        assert!(
            preview(d.path(), "absorb")
                .unwrap()
                .contains(&"AGENTS.md".to_string())
        );
        assert!(preview(d.path(), "rm -rf").is_err());
    }

    #[test]
    fn preview_absorb_lists_targets_it_overwrites() {
        let d = adopted();
        let claude = d.path().join("CLAUDE.md");
        std::fs::write(
            &claude,
            std::fs::read_to_string(&claude).unwrap() + "edit\n",
        )
        .unwrap();
        let listed = preview(d.path(), "absorb").unwrap();
        assert!(listed.contains(&"CLAUDE.md".to_string()), "{listed:?}");
    }

    #[test]
    fn run_refuses_when_files_changed_since_review() {
        let d = adopted();
        std::fs::write(d.path().join("CLAUDE.md"), "edit\n").unwrap();
        let listed = preview(d.path(), "discard").unwrap();
        // An agent edits another target while the dialog is open.
        let settings = d.path().join(".claude/settings.json");
        std::fs::write(&settings, "{}\n").unwrap();
        let err = run(d.path(), "discard", &listed).unwrap_err();
        assert!(err.contains("review again"), "{err}");
        assert_eq!(std::fs::read_to_string(&settings).unwrap(), "{}\n");
        assert_eq!(
            std::fs::read_to_string(d.path().join("CLAUDE.md")).unwrap(),
            "edit\n"
        );
    }

    #[test]
    fn missing_folder_offers_nothing_and_actions_refuse() {
        let d = tempfile::tempdir().unwrap();
        let gone = d.path().join("moved-away");
        let s = status(&gone, None);
        assert!(s.offer.is_none() && !s.has_config, "{s:?}");
        assert!(s.error.unwrap().contains("not found"));
        for action in ["init", "hooks", "import"] {
            assert!(run(&gone, action, &[]).is_err(), "{action}");
        }
        assert!(!gone.exists(), "an action recreated the folder");
    }

    #[test]
    fn run_discard_restores_and_hooks_installs_only_hooks() {
        let d = adopted();
        std::fs::write(d.path().join("CLAUDE.md"), "edit\n").unwrap();
        run(d.path(), "discard", &["CLAUDE.md".to_string()]).unwrap();
        assert!(status(d.path(), None).drift.is_empty());

        let e = repo();
        run(e.path(), "hooks", &[]).unwrap();
        assert!(!e.path().join("tempa.toml").exists());
        assert!(status(e.path(), None).hooks_ok);
    }

    #[test]
    fn cli_version_parses_tempa_version_output() {
        assert_eq!(parse_cli_version("tempa 0.1.0\n").as_deref(), Some("0.1.0"));
        assert_eq!(parse_cli_version("garbage"), None);
    }
}
