//! `tempa check`: drift + document contract + per-PR task rules + harness.

use crate::config::Config;
use crate::contract::{check_docs, parse_changes};
use crate::gitx;
use crate::sync::{LOCK, Lock, drift, drift_findings};
use crate::{Error, Finding, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct CheckOpts {
    pub contract_only: bool,
    pub base: Option<String>,
}

const MANIFESTS: [&str; 10] = [
    "package.json",
    "Cargo.toml",
    "go.mod",
    "pyproject.toml",
    "requirements.txt",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "composer.json",
    "Gemfile",
];

fn globs(patterns: &[String]) -> Result<GlobSet> {
    let mut b = GlobSetBuilder::new();
    for p in patterns {
        b.add(Glob::new(p).map_err(|e| Error(format!("glob tidak valid {p}: {e}")))?);
    }
    b.build().map_err(|e| Error(e.to_string()))
}

fn dir(d: &str) -> String {
    format!("{}/", d.trim_end_matches('/'))
}

pub fn check(root: &Path, opts: &CheckOpts) -> Result<Vec<Finding>> {
    let cfg = Config::load(root)?;
    let mut f = Vec::new();

    if let Some(v) = &cfg.project.tempa_version {
        if v != env!("CARGO_PKG_VERSION") {
            f.push(Finding::error(
                "version.mismatch",
                Some(crate::config::FILE.into()),
                format!("tempa.toml mengunci versi {v}, binary ini {}", env!("CARGO_PKG_VERSION")),
            ));
        }
    }

    f.extend(drift_findings(&drift(root)?));

    let base_name = gitx::base_name(&cfg, opts.base.as_deref());
    let base = gitx::resolve_base(root, &cfg, opts.base.as_deref());
    if base.is_none() {
        let msg = format!(
            "branch dasar {base_name} tidak ditemukan atau riwayat tidak cukup; aturan task dilewati. \
             Di Bitbucket Pipelines pakai `clone: depth: full`"
        );
        f.push(if std::env::var("BITBUCKET_PR_ID").is_ok_and(|v| !v.is_empty()) {
            Finding::error("git.base_unavailable", None, msg)
        } else {
            Finding::warning("git.base_unavailable", None, msg)
        });
    }
    let on_base = gitx::current_branch(root).is_ok_and(|b| b == base_name);
    let range = base.filter(|_| !on_base);

    let skip = match &range {
        Some(b) => {
            let authors = gitx::authors(root, b)?;
            !authors.is_empty() && authors.iter().all(|a| cfg.ci.skip_authors.contains(a))
        }
        None => false,
    };

    if !skip {
        f.extend(check_docs(root, &cfg)?);
        if let Some(b) = &range {
            let changed = gitx::changed_files(root, b)?;
            let docs_only = globs(&cfg.ci.docs_only_paths)?;
            if !changed.is_empty() && !changed.iter().all(|p| docs_only.is_match(p)) {
                f.extend(task_findings(root, &cfg, b)?);
            }
        }
    }

    if !opts.contract_only {
        f.extend(run_harness(root, &cfg)?);
    }
    Ok(f)
}

/// Per-PR rules for the task this branch implements.
pub fn task_findings(root: &Path, cfg: &Config, base: &str) -> Result<Vec<Finding>> {
    let mut f = Vec::new();
    let changed = gitx::changed_files(root, base)?;
    let plan_dir = dir(&cfg.docs.plan);
    let branch = gitx::current_branch(root).unwrap_or_default();

    let id = if let Some(id) = branch.strip_prefix("tempa/") {
        let plan = format!("{plan_dir}{id}.md");
        if !root.join(&plan).is_file() {
            f.push(Finding::error("task.no_plan", Some(plan), format!("branch {branch} belum punya file plan")));
        }
        Some(id.to_string())
    } else {
        let plans: Vec<&String> = changed.iter().filter(|p| p.starts_with(&plan_dir) && p.ends_with(".md")).collect();
        match plans.as_slice() {
            [] => {
                f.push(Finding::error("task.no_plan", None, format!("PR ini tidak menambah/mengubah plan di {plan_dir}")));
                None
            }
            [one] => Some(one[plan_dir.len()..one.len() - 3].to_string()),
            many => {
                let list = many.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                f.push(Finding::error("task.multiple_plans", None, format!("satu PR satu task; plan berubah: {list}")));
                None
            }
        }
    };

    if let Some(id) = &id {
        let changes = std::fs::read_to_string(root.join(&cfg.docs.changes)).unwrap_or_default();
        if !parse_changes(&changes).iter().any(|e| &e.task_id == id) {
            f.push(Finding::error(
                "task.no_change_entry",
                Some(cfg.docs.changes.clone()),
                format!("belum ada entri changes untuk task {id}; jalankan tempa log"),
            ));
        }
    }

    let g = &cfg.gates;
    let (schema, api) = (globs(&g.schema_paths)?, globs(&g.public_api_paths)?);
    let file_name = |p: &str| p.rsplit('/').next().unwrap_or(p).to_string();
    let triggers: Vec<&str> = g
        .decision_required_for
        .iter()
        .map(String::as_str)
        .filter(|t| match *t {
            "new-dependency" => changed.iter().any(|p| MANIFESTS.contains(&file_name(p).as_str())),
            "schema-change" => changed.iter().any(|p| schema.is_match(p)),
            "public-api" => changed.iter().any(|p| api.is_match(p)),
            _ => false,
        })
        .collect();
    if !triggers.is_empty() {
        let dec_dir = dir(&cfg.docs.decisions);
        if !gitx::added_files(root, base)?.iter().any(|p| p.starts_with(&dec_dir) && p.ends_with(".md")) {
            f.push(Finding::error(
                "task.decision_required",
                None,
                format!("perubahan memicu {}; tambahkan decision baru (tempa decide)", triggers.join(", ")),
            ));
        }
    }

    let mut exclude: BTreeSet<String> = Lock::load(root).files.into_keys().collect();
    exclude.insert(LOCK.to_string());
    let lines = gitx::diff_lines(root, base, &exclude)?;
    if lines > g.max_diff_lines {
        f.push(Finding::error(
            "task.diff_too_large",
            None,
            format!("diff {lines} baris melebihi max_diff_lines {}; pecah task", g.max_diff_lines),
        ));
    }
    Ok(f)
}

/// Run `[harness]` commands in order; stop at the first failure.
pub fn run_harness(root: &Path, cfg: &Config) -> Result<Vec<Finding>> {
    for (name, cmd) in &cfg.harness {
        let status = if cfg!(windows) {
            Command::new("cmd").args(["/C", cmd]).current_dir(root).status()
        } else {
            Command::new("sh").args(["-c", cmd]).current_dir(root).status()
        }
        .map_err(|e| Error(format!("harness {name} tidak bisa dijalankan: {e}")))?;
        if !status.success() {
            let code = status.code().map(|c| c.to_string()).unwrap_or_else(|| "signal".into());
            return Ok(vec![Finding::error("harness.failed", Some(name.clone()), format!("`{cmd}` gagal (exit {code})"))]);
        }
    }
    Ok(vec![])
}
