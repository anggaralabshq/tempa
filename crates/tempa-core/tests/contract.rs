mod common;
use common::write;
use tempa_core::config::{Config, default_toml};
use tempa_core::contract::{CHANGE_FIELDS, check_docs, parse_changes, require_headings};

fn v(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

const ENTRY: &str = "## 2026-11-06 · 20261106-add-invoice
- Ringkasan: Tambah endpoint invoice
- Alasan: Permintaan tim finance
- Decision: docs/decisions/2026-11-06-invoice-numbering.md
- Harness: lulus
- Agent: codex / gpt-5
";

const HARNESS: &str = "## Definisi selesai\n\nTest lulus.\n\n## Gate yang berlaku\n\nSemua.\n";
const AGENTS: &str = "## Stack\nx\n## Perintah\nx\n## Konvensi\nx\n## Area terlarang\nx\n";

fn codes(f: &[tempa_core::Finding]) -> Vec<&'static str> {
    f.iter().map(|x| x.code).collect()
}

#[test]
fn heading_missing_and_empty() {
    let r = require_headings(
        "## Stack\nRust\n## Perintah\n\n",
        &v(&["Stack", "Perintah", "Area terlarang"]),
    );
    assert_eq!(
        r,
        vec![
            ("Perintah".to_string(), "empty"),
            ("Area terlarang".to_string(), "missing")
        ]
    );
}

#[test]
fn heading_match_case_insensitive_any_level() {
    assert!(require_headings("### stack\nx\n", &v(&["Stack"])).is_empty());
}

#[test]
fn heading_with_only_subheading_is_not_empty() {
    assert!(require_headings("## Langkah\n### Satu\n", &v(&["Langkah"])).is_empty());
}

#[test]
fn parses_change_entries() {
    let e = parse_changes(&format!(
        "# Changes\n\n{ENTRY}\n## 2026-11-07 · t2\n- Ringkasan: x\n"
    ));
    assert_eq!(e.len(), 2);
    assert_eq!(e[0].date, "2026-11-06");
    assert_eq!(e[0].task_id, "20261106-add-invoice");
    assert_eq!(e[0].fields.len(), CHANGE_FIELDS.len());
    assert_eq!(
        e[0].fields[4],
        ("Agent".to_string(), "codex / gpt-5".to_string())
    );
    assert_eq!(e[1].task_id, "t2");
}

#[test]
fn valid_repo_has_no_findings() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    write(p, "tempa.toml", &default_toml("x"));
    write(p, "AGENTS.md", AGENTS);
    write(p, "HARNESS.md", HARNESS);
    write(p, "docs/plans/.gitkeep", "");
    write(
        p,
        "docs/decisions/2026-11-06-invoice-numbering.md",
        "## Konteks\na\n## Opsi\nb\n## Keputusan\nc\n## Konsekuensi\nd\n## Agent + model\ne\n",
    );
    write(p, "CHANGES.md", &format!("# Changes\n\n{ENTRY}"));
    let f = check_docs(p, &Config::load(p).unwrap()).unwrap();
    assert!(f.is_empty(), "{f:?}");
}

#[test]
fn check_docs_reports_each_role() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    write(p, "tempa.toml", &default_toml("x"));
    write(p, "AGENTS.md", "## Stack\nx\n");
    write(p, "HARNESS.md", HARNESS);
    write(p, "docs/plans/t1.md", "## Tujuan\nx\n");
    write(
        p,
        "CHANGES.md",
        "## 2026-11-06 · t1\n- Ringkasan: a\n- Alasan: b\n- Decision: docs/decisions/tidak-ada.md\n- Harness: lulus\n",
    );
    let f = check_docs(p, &Config::load(p).unwrap()).unwrap();
    let c = codes(&f);
    assert!(c.contains(&"contract.missing_heading"), "{f:?}");
    assert!(c.contains(&"contract.plan_frontmatter"), "{f:?}");
    assert!(c.contains(&"contract.decision_link_broken"), "{f:?}");
    assert!(c.contains(&"contract.change_field"), "{f:?}");
    assert!(
        f.iter()
            .any(|x| x.path.as_deref() == Some("docs/plans/t1.md"))
    );
    assert!(f.iter().all(|x| x.severity == tempa_core::Severity::Error));
}

#[test]
fn missing_docs_reported() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "tempa.toml", &default_toml("x"));
    let f = check_docs(d.path(), &Config::load(d.path()).unwrap()).unwrap();
    let missing: Vec<_> = f
        .iter()
        .filter(|x| x.code == "contract.missing_doc")
        .filter_map(|x| x.path.clone())
        .collect();
    assert_eq!(missing, vec!["AGENTS.md", "HARNESS.md", "CHANGES.md"]);
}

#[test]
fn custom_headings_override() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    write(
        p,
        "tempa.toml",
        &(default_toml("x") + "\n[headings]\ncontext = [\"Stack\"]\n"),
    );
    write(p, "AGENTS.md", "## Stack\nx\n");
    let f = check_docs(p, &Config::load(p).unwrap()).unwrap();
    assert!(
        !f.iter().any(|x| x.path.as_deref() == Some("AGENTS.md")),
        "{f:?}"
    );
}
