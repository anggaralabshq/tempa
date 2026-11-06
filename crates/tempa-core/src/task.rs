//! Out-of-app workflow: `tempa task new|approve`, `decide`, `log`, `pr-body`.

use crate::config::Config;
use crate::contract::{CHANGE_FIELDS, ChangeEntry, parse_changes};
use crate::date::{compact_today, iso_today};
use crate::md::{sections, set_front, split_front};
use crate::{Error, Result, gitx, write_atomic};
use std::path::Path;

const PLAN: &str = include_str!("templates/plan.md");
const DECISION: &str = include_str!("templates/decision.md");

fn dir(d: &str) -> String {
    format!("{}/", d.trim_end_matches('/'))
}

fn valid_slug(s: &str) -> bool {
    !s.is_empty()
        && s.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

fn check_slug(slug: &str) -> Result<()> {
    if valid_slug(slug) {
        Ok(())
    } else {
        Err(Error(format!(
            "invalid slug \"{slug}\": use lowercase letters, digits and hyphens, e.g. add-invoice"
        )))
    }
}

pub fn plan_path(cfg: &Config, id: &str) -> String {
    format!("{}{id}.md", dir(&cfg.docs.plan))
}

/// Create branch `tempa/<YYYYMMDD-slug>` and its plan file. Returns the task id.
pub fn task_new(root: &Path, slug: &str) -> Result<String> {
    check_slug(slug)?;
    let cfg = Config::load(root)?;
    if !gitx::is_repo(root) {
        return Err(Error("not a git repository; run `git init` first".into()));
    }
    let id = format!("{}-{slug}", compact_today());
    let path = plan_path(&cfg, &id);
    if root.join(&path).exists() {
        return Err(Error(format!("{path} already exists")));
    }
    let from = gitx::current_branch(root).ok();
    gitx::switch_new(root, &format!("tempa/{id}"))?;
    if let Some(from) = from {
        gitx::set_task_base(root, &from)?;
    }
    write_atomic(&root.join(&path), PLAN.replace("{id}", &id).as_bytes())?;
    Ok(id)
}

pub fn current_task_id(root: &Path) -> Result<String> {
    let branch = gitx::current_branch(root)?;
    branch
        .strip_prefix("tempa/")
        .map(String::from)
        .ok_or_else(|| Error(format!("{branch} is not a task branch; use tempa task new")))
}

pub fn task_approve(root: &Path) -> Result<()> {
    let cfg = Config::load(root)?;
    let path = plan_path(&cfg, &current_task_id(root)?);
    let text = std::fs::read_to_string(root.join(&path))
        .map_err(|_| Error(format!("{path} not found")))?;
    let text = set_front(&text, "state", "approved");
    let text = set_front(&text, "approved_by", &gitx::user_name(root)?);
    let text = set_front(&text, "approved_at", &iso_today());
    write_atomic(&root.join(&path), text.as_bytes())
}

/// Create `<decisions>/<YYYY-MM-DD>-<slug>.md`. Returns its path.
pub fn decide(root: &Path, slug: &str) -> Result<String> {
    check_slug(slug)?;
    let cfg = Config::load(root)?;
    let path = format!("{}{}-{slug}.md", dir(&cfg.docs.decisions), iso_today());
    if root.join(&path).exists() {
        return Err(Error(format!("{path} already exists")));
    }
    write_atomic(&root.join(&path), DECISION.as_bytes())?;
    Ok(path)
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub summary: String,
    pub reason: String,
    pub decision: Option<String>,
    /// `pass` | `fail` | `pending`
    pub harness: String,
    pub agent: String,
}

fn one_line(name: &str, v: &str) -> Result<()> {
    if v.trim().is_empty() || v.contains('\n') {
        return Err(Error(format!("{name} is required, on one line")));
    }
    Ok(())
}

fn render_entry(e: &ChangeEntry) -> String {
    let mut s = format!("## {} · {}\n", e.date, e.task_id);
    for (k, v) in &e.fields {
        s += &format!("- {k}: {v}\n");
    }
    s
}

/// Append a changes entry for the current task.
pub fn log(root: &Path, e: &LogEntry) -> Result<()> {
    one_line("--summary", &e.summary)?;
    one_line("--reason", &e.reason)?;
    one_line("--agent", &e.agent)?;
    // Indonesian values from tempa 0.1 are accepted and normalized.
    let harness = match e.harness.as_str() {
        "pass" | "lulus" => "pass",
        "fail" | "gagal" => "fail",
        "pending" | "belum" => "pending",
        other => {
            return Err(Error(format!(
                "--harness must be pass, fail or pending (got \"{other}\")"
            )));
        }
    };
    let decision = e.decision.clone().unwrap_or_else(|| "-".into());
    one_line("--decision", &decision)?;
    let cfg = Config::load(root)?;
    let entry = ChangeEntry {
        date: iso_today(),
        task_id: current_task_id(root)?,
        fields: CHANGE_FIELDS
            .iter()
            .zip([e.summary.as_str(), &e.reason, &decision, harness, &e.agent])
            .map(|(k, v)| (k.to_string(), v.trim().to_string()))
            .collect(),
    };
    let path = root.join(&cfg.docs.changes);
    let mut text = std::fs::read_to_string(&path).unwrap_or_else(|_| "# Changes\n".into());
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text += &format!("\n{}", render_entry(&entry));
    write_atomic(&path, text.as_bytes())
}

/// PR description for the current task, headings in spec 9.2 order.
pub fn pr_body(root: &Path) -> Result<String> {
    let cfg = Config::load(root)?;
    let id = current_task_id(root)?;
    let path = plan_path(&cfg, &id);
    let plan = std::fs::read_to_string(root.join(&path)).unwrap_or_default();
    let summary = sections(&plan)
        .into_iter()
        .find(|s| s.title.eq_ignore_ascii_case("Tujuan"))
        .map(|s| s.body.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".into());
    let reviewer = split_front(&plan)
        .0
        .and_then(|f| {
            f.pairs
                .into_iter()
                .find(|(k, v)| k == "reviewer" && !v.is_empty())
                .map(|(_, v)| v)
        })
        .unwrap_or_else(|| "-".into());

    let dec_dir = dir(&cfg.docs.decisions);
    let decisions: Vec<String> = gitx::resolve_base(root, &cfg, None)
        .and_then(|b| gitx::added_files(root, &b).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.starts_with(&dec_dir) && p.ends_with(".md"))
        .collect();

    let changes = std::fs::read_to_string(root.join(&cfg.docs.changes)).unwrap_or_default();
    let entries: Vec<ChangeEntry> = parse_changes(&changes)
        .into_iter()
        .filter(|e| e.task_id == id)
        .collect();
    let field = |name: &str| {
        entries
            .last()
            .and_then(|e| {
                e.fields
                    .iter()
                    .find(|(k, _)| k == name)
                    .map(|(_, v)| v.clone())
            })
            .unwrap_or_else(|| "-".into())
    };
    let agent = field("Agent");
    let harness = format!(
        "{} (recorded with `tempa log`). Re-verify: `tempa check --base {}`",
        field("Harness"),
        cfg.ci.base
    );
    let or_dash = |v: Vec<String>| {
        if v.is_empty() {
            "-".to_string()
        } else {
            v.join("\n")
        }
    };

    Ok(format!(
        "## Ringkasan\n\n{summary}\n\n## Link plan\n\n{path}\n\n## Decision terkait\n\n{}\n\n\
         ## Entri changes\n\n{}\n\n## Hasil harness\n\n{harness}\n\n\
         ## Review\n\n{reviewer}\n\n## Agent + model\n\n{agent}\n",
        or_dash(decisions.iter().map(|d| format!("- {d}")).collect()),
        or_dash(
            entries
                .iter()
                // Entry headings are `##`; demote so the PR keeps its own sections.
                .map(|e| format!("#{}", render_entry(e).trim_end()))
                .collect()
        ),
    ))
}
