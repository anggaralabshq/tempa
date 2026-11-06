//! `tempa doctor`: per-agent parity report. A report, never a gate.

use crate::config::{Agent, Config};
use crate::targets::{Gap, render};
use crate::{Result, gitx, init::HOOKS_DIR, source, sync};
use std::path::Path;

#[derive(Debug)]
pub struct AgentRow {
    pub agent: Agent,
    pub files: usize,
    pub gaps: Vec<Gap>,
    /// "settings deny" | "gap (CI)"
    pub protected: &'static str,
    /// Lifecycle hooks arrive with tempa_context in M3.
    pub lifecycle: &'static str,
}

#[derive(Debug)]
pub struct DoctorReport {
    pub rows: Vec<AgentRow>,
    pub legacy: Vec<String>,
    pub hooks_ok: bool,
    pub drift: usize,
}

const LEGACY: [&str; 3] = [
    "GEMINI.md",
    ".cursorrules",
    ".github/copilot-instructions.md",
];

pub fn doctor(root: &Path) -> Result<DoctorReport> {
    let cfg = Config::load(root)?;
    let src = source::load(root, &cfg)?;
    let rows = cfg
        .agents
        .enabled
        .iter()
        .map(|&agent| {
            let r = render(agent, &cfg, &src);
            AgentRow {
                agent,
                files: r.files.len(),
                gaps: r.gaps,
                // ponytail: only Claude has a project-level deny list today (docs/target-matrix.md).
                protected: if agent == Agent::Claude {
                    "settings deny"
                } else {
                    "gap (CI)"
                },
                lifecycle: "M3",
            }
        })
        .collect();

    let mut legacy: Vec<String> = LEGACY
        .iter()
        .filter(|p| root.join(p).is_file())
        .map(|p| p.to_string())
        .collect();
    for (rel, _) in source::walk(&root.join(".cursor/rules"))? {
        if rel.ends_with(".mdc") && rel != "tempa.mdc" {
            legacy.push(format!(".cursor/rules/{rel}"));
        }
    }

    let hooks_ok = gitx::is_repo(root)
        && gitx::get_config(root, "core.hooksPath").as_deref() == Some(HOOKS_DIR);
    Ok(DoctorReport {
        rows,
        legacy,
        hooks_ok,
        drift: sync::drift(root)?.len(),
    })
}
