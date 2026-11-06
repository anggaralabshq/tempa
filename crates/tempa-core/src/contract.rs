//! Structural checks for the contract documents (spec 9). Structure only, never style.

use crate::config::Config;
use crate::md::{sections, split_front};
use crate::source::walk;
use crate::{Finding, Result};
use std::path::Path;

pub const CHANGE_FIELDS: [&str; 5] = ["Summary", "Reason", "Decision", "Harness", "Agent"];

/// Indonesian vocabulary of tempa 0.1 (default headings and change fields), still accepted
/// so repos written before 1.0 keep passing.
const LEGACY: [(&str, &str); 15] = [
    ("Commands", "Perintah"),
    ("Conventions", "Konvensi"),
    ("Off-limits", "Area terlarang"),
    ("Goal", "Tujuan"),
    ("Steps", "Langkah"),
    ("Files touched", "File yang disentuh"),
    ("Risks", "Risiko"),
    ("Context", "Konteks"),
    ("Options", "Opsi"),
    ("Decision", "Keputusan"),
    ("Consequences", "Konsekuensi"),
    ("Definition of done", "Definisi selesai"),
    ("Gates", "Gate yang berlaku"),
    ("Summary", "Ringkasan"),
    ("Reason", "Alasan"),
];

/// `have` names the heading or field `want` (case-insensitive, or its 0.1 Indonesian name).
pub fn same_name(want: &str, have: &str) -> bool {
    have.eq_ignore_ascii_case(want)
        || LEGACY
            .iter()
            .any(|(en, id)| want.eq_ignore_ascii_case(en) && have.eq_ignore_ascii_case(id))
}

/// Required headings that are missing or empty: `(heading, "missing" | "empty")`.
/// Case-insensitive, any heading level; a body with only sub-headings is not empty.
pub fn require_headings(md: &str, required: &[String]) -> Vec<(String, &'static str)> {
    let secs = sections(md);
    required
        .iter()
        .filter_map(|h| match secs.iter().find(|s| same_name(h, &s.title)) {
            None => Some((h.clone(), "missing")),
            Some(s) if s.body.trim().is_empty() => Some((h.clone(), "empty")),
            Some(_) => None,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeEntry {
    pub date: String,
    pub task_id: String,
    pub fields: Vec<(String, String)>,
}

/// Entries headed `## <YYYY-MM-DD> · <task-id>` with `- <Name>: <value>` lines.
pub fn parse_changes(md: &str) -> Vec<ChangeEntry> {
    let mut out: Vec<ChangeEntry> = Vec::new();
    for line in crate::md::normalize_eol(md).lines() {
        if let Some(head) = line.strip_prefix("## ") {
            if let Some((date, id)) = head.split_once('·') {
                out.push(ChangeEntry {
                    date: date.trim().into(),
                    task_id: id.trim().into(),
                    fields: vec![],
                });
                continue;
            }
        }
        if let (Some(e), Some(item)) = (out.last_mut(), line.strip_prefix("- ")) {
            if let Some((k, v)) = item.split_once(':') {
                e.fields.push((k.trim().into(), v.trim().into()));
            }
        }
    }
    out
}

fn headings(findings: &mut Vec<Finding>, path: &str, md: &str, required: &[String]) {
    for (h, kind) in require_headings(md, required) {
        findings.push(if kind == "missing" {
            Finding::error(
                "contract.missing_heading",
                Some(path.into()),
                format!("required heading \"{h}\" is missing"),
            )
        } else {
            Finding::error(
                "contract.empty_heading",
                Some(path.into()),
                format!("heading \"{h}\" is empty"),
            )
        });
    }
}

fn doc(root: &Path, findings: &mut Vec<Finding>, path: &str) -> Option<String> {
    match std::fs::read_to_string(root.join(path)) {
        Ok(t) => Some(t),
        Err(_) => {
            findings.push(Finding::error(
                "contract.missing_doc",
                Some(path.into()),
                "contract document missing; run tempa init",
            ));
            None
        }
    }
}

fn md_files(root: &Path, dir: &str) -> Result<Vec<(String, String)>> {
    let dir = dir.trim_end_matches('/');
    walk(&root.join(dir))?
        .into_iter()
        .filter(|(rel, _)| rel.ends_with(".md"))
        .map(|(rel, p)| Ok((format!("{dir}/{rel}"), std::fs::read_to_string(p)?)))
        .collect()
}

pub fn check_docs(root: &Path, cfg: &Config) -> Result<Vec<Finding>> {
    let mut f = Vec::new();
    let (docs, h) = (&cfg.docs, &cfg.headings);

    if let Some(t) = doc(root, &mut f, &docs.context) {
        headings(&mut f, &docs.context, &t, &h.context);
    }
    if let Some(t) = doc(root, &mut f, &docs.harness) {
        headings(&mut f, &docs.harness, &t, &h.harness);
    }

    for (path, text) in md_files(root, &docs.plan)? {
        let (front, _) = split_front(&text);
        let has = |k: &str| {
            front
                .as_ref()
                .is_some_and(|fr| fr.pairs.iter().any(|(key, v)| key == k && !v.is_empty()))
        };
        if !has("task_id") || !has("state") {
            f.push(Finding::error(
                "contract.plan_frontmatter",
                Some(path.clone()),
                "plan frontmatter must contain task_id and state",
            ));
        }
        headings(&mut f, &path, &text, &h.plan);
    }

    for (path, text) in md_files(root, &docs.decisions)? {
        headings(&mut f, &path, &text, &h.decisions);
    }

    if let Some(t) = doc(root, &mut f, &docs.changes) {
        for e in parse_changes(&t) {
            let at = Some(docs.changes.clone());
            for field in CHANGE_FIELDS {
                match e.fields.iter().find(|(k, _)| same_name(field, k)) {
                    Some((_, v)) if !v.is_empty() => {
                        if field == "Decision" && v != "-" && !root.join(v).is_file() {
                            f.push(Finding::error(
                                "contract.decision_link_broken",
                                at.clone(),
                                format!("entry {}: decision {v} does not exist", e.task_id),
                            ));
                        }
                    }
                    _ => f.push(Finding::error(
                        "contract.change_field",
                        at.clone(),
                        format!(
                            "entry {}: field \"{field}\" is required (use \"-\" for no Decision)",
                            e.task_id
                        ),
                    )),
                }
            }
        }
    }
    Ok(f)
}
