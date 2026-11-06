mod common;
use common::{git, read, valid_repo, write};
use tempa_core::check::{CheckOpts, check};
use tempa_core::contract::parse_changes;
use tempa_core::gitx::current_branch;
use tempa_core::task::{LogEntry, current_task_id, decide, log, pr_body, task_approve, task_new};

const PLAN_BODY: &str = "## Tujuan\nTambah invoice\n## Langkah\n- satu\n## File yang disentuh\nsrc/invoice.rs\n## Risiko\nrendah\n";

fn entry(harness: &str) -> LogEntry {
    LogEntry {
        summary: "Tambah endpoint invoice".into(),
        reason: "Permintaan finance".into(),
        decision: None,
        harness: harness.into(),
        agent: "codex / gpt-5".into(),
    }
}

fn fill_plan(p: &std::path::Path, id: &str) {
    let path = format!("docs/plans/{id}.md");
    let text = read(p, &path);
    let front_end = text.find("---\n## ").unwrap() + 4;
    std::fs::write(p.join(&path), format!("{}{PLAN_BODY}", &text[..front_end])).unwrap();
}

#[test]
fn task_new_creates_branch_and_plan() {
    let d = valid_repo();
    let id = task_new(d.path(), "add-invoice").unwrap();
    assert!(id.ends_with("-add-invoice") && id.len() == "YYYYMMDD-add-invoice".len(), "{id}");
    assert_eq!(current_branch(d.path()).unwrap(), format!("tempa/{id}"));
    assert_eq!(current_task_id(d.path()).unwrap(), id);
    let plan = read(d.path(), &format!("docs/plans/{id}.md"));
    assert!(plan.starts_with(&format!("---\ntask_id: {id}\nstate: plan\n")), "{plan}");
    for h in ["## Tujuan", "## Langkah", "## File yang disentuh", "## Risiko"] {
        assert!(plan.contains(h));
    }
}

#[test]
fn bad_slug_rejected() {
    let d = valid_repo();
    for slug in ["Add Invoice", "-x", "x-", "a--b", ""] {
        let e = task_new(d.path(), slug).unwrap_err();
        assert!(e.0.contains("add-invoice"), "{slug}: {}", e.0);
    }
}

#[test]
fn current_task_id_off_task_branch_errors() {
    let d = valid_repo();
    assert!(current_task_id(d.path()).unwrap_err().0.contains("tempa task new"));
}

#[test]
fn approve_sets_frontmatter() {
    let d = valid_repo();
    let id = task_new(d.path(), "x").unwrap();
    task_approve(d.path()).unwrap();
    let plan = read(d.path(), &format!("docs/plans/{id}.md"));
    assert!(plan.contains("state: approved\n") && plan.contains("approved_by: test\n"), "{plan}");
    assert!(plan.contains(&format!("approved_at: {}\n", tempa_core::date::iso_today())));
}

#[test]
fn decide_creates_once() {
    let d = valid_repo();
    let p = decide(d.path(), "invoice-numbering").unwrap();
    assert_eq!(p, format!("docs/decisions/{}-invoice-numbering.md", tempa_core::date::iso_today()));
    assert!(read(d.path(), &p).contains("## Agent + model"));
    assert!(decide(d.path(), "invoice-numbering").is_err());
}

#[test]
fn log_appends_valid_entry() {
    let d = valid_repo();
    let id = task_new(d.path(), "x").unwrap();
    log(d.path(), &entry("belum")).unwrap();
    log(d.path(), &entry("lulus")).unwrap();
    let e = parse_changes(&read(d.path(), "CHANGES.md"));
    assert_eq!(e.len(), 2);
    assert!(e.iter().all(|x| x.task_id == id));
    assert!(e[0].fields.contains(&("Harness".to_string(), "belum".to_string())));
    assert!(e[0].fields.contains(&("Decision".to_string(), "-".to_string())));
}

#[test]
fn log_rejects_bad_values() {
    let d = valid_repo();
    task_new(d.path(), "x").unwrap();
    assert!(log(d.path(), &entry("ok")).is_err());
    let mut e = entry("lulus");
    e.summary = " ".into();
    assert!(log(d.path(), &e).is_err());
    let mut e = entry("lulus");
    e.reason = "dua\nbaris".into();
    assert!(log(d.path(), &e).is_err());
}

#[test]
fn pr_body_sections_in_order() {
    let d = valid_repo();
    let id = task_new(d.path(), "x").unwrap();
    fill_plan(d.path(), &id);
    log(d.path(), &entry("lulus")).unwrap();
    let b = pr_body(d.path()).unwrap();
    let heads = ["## Ringkasan", "## Link plan", "## Decision terkait", "## Entri changes", "## Hasil harness", "## Review", "## Agent + model"];
    let pos: Vec<_> = heads.iter().map(|h| b.find(h).unwrap_or_else(|| panic!("{h} missing in {b}"))).collect();
    assert!(pos.windows(2).all(|w| w[0] < w[1]), "{b}");
    assert!(b.contains("Tambah invoice"));
    assert!(b.contains(&format!("docs/plans/{id}.md")));
    assert!(b.contains("codex / gpt-5"));
    assert!(b.contains("Lihat pipeline Bitbucket."));
}

#[test]
fn full_cli_flow_passes_check() {
    let d = valid_repo();
    let id = task_new(d.path(), "add-invoice").unwrap();
    write(d.path(), "src/invoice.rs", "pub fn invoice() {}\n");
    fill_plan(d.path(), &id);
    task_approve(d.path()).unwrap();
    log(d.path(), &entry("lulus")).unwrap();
    git(d.path(), &["add", "-A"]);
    git(d.path(), &["commit", "-qm", "task"]);
    let f = check(d.path(), &CheckOpts { contract_only: true, base: None }).unwrap();
    assert!(f.iter().all(|x| x.severity != tempa_core::Severity::Error), "{f:?}");
}
