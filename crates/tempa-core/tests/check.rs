mod common;
use common::{git, git_init, read, write};
use std::path::Path;
use tempa_core::check::{CheckOpts, check, run_harness};
use tempa_core::config::{Config, default_toml};
use tempa_core::{Finding, Severity};

const AGENTS: &str = "## Stack\nx\n## Perintah\nx\n## Konvensi\nx\n## Area terlarang\nx\n";
const HARNESS: &str = "## Definisi selesai\nx\n## Gate yang berlaku\nx\n";
const PLAN: &str = "## Tujuan\nx\n## Langkah\nx\n## File yang disentuh\nx\n## Risiko\nx\n";
const DECISION: &str =
    "## Konteks\na\n## Opsi\nb\n## Keputusan\nc\n## Konsekuensi\nd\n## Agent + model\ne\n";

fn contract() -> CheckOpts {
    CheckOpts {
        contract_only: true,
        base: None,
    }
}

fn codes(f: &[Finding]) -> Vec<&'static str> {
    f.iter().map(|x| x.code).collect()
}

fn errors(f: &[Finding]) -> Vec<&Finding> {
    f.iter().filter(|x| x.severity == Severity::Error).collect()
}

fn commit(p: &Path, msg: &str) {
    git(p, &["add", "-A"]);
    git(p, &["commit", "-qm", msg]);
}

/// Valid repo committed on `main`, then switched to `branch`.
fn repo_on(branch: &str) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    git_init(p);
    write(p, "tempa.toml", &default_toml("x"));
    write(p, "AGENTS.md", AGENTS);
    write(p, "HARNESS.md", HARNESS);
    write(p, "CHANGES.md", "# Changes\n");
    write(p, "docs/plans/.gitkeep", "");
    write(p, "docs/decisions/.gitkeep", "");
    tempa_core::sync::sync(p).unwrap();
    commit(p, "init");
    git(p, &["switch", "-qc", branch]);
    d
}

fn add_plan_and_change(p: &Path, id: &str) {
    write(
        p,
        &format!("docs/plans/{id}.md"),
        &format!("---\ntask_id: {id}\nstate: plan\n---\n{PLAN}"),
    );
    let entry = format!(
        "\n## 2026-11-06 · {id}\n- Ringkasan: s\n- Alasan: r\n- Decision: -\n- Harness: belum\n- Agent: codex / gpt-5\n"
    );
    std::fs::write(p.join("CHANGES.md"), read(p, "CHANGES.md") + &entry).unwrap();
}

#[test]
fn clean_main_has_no_errors() {
    let d = repo_on("main-copy");
    git(d.path(), &["switch", "-q", "main"]);
    let f = check(d.path(), &contract()).unwrap();
    assert!(errors(&f).is_empty(), "{f:?}");
}

#[test]
fn missing_base_is_warning_locally() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    git(p, &["init", "-q", "-b", "work"]);
    git(p, &["config", "user.name", "t"]);
    git(p, &["config", "user.email", "t@x"]);
    write(p, "tempa.toml", &default_toml("x"));
    let f = check(p, &contract()).unwrap();
    let b = f
        .iter()
        .find(|x| x.code == "git.base_unavailable")
        .expect("base finding");
    assert_eq!(b.severity, Severity::Warning);
    assert!(b.message.contains("depth: full"));
    assert!(
        codes(&f).contains(&"contract.missing_doc"),
        "docs still checked without a range: {f:?}"
    );
}

#[test]
fn tempa_branch_requires_plan_and_change() {
    let d = repo_on("tempa/20261106-x");
    write(d.path(), "src/a.rs", "fn a() {}\n");
    commit(d.path(), "code");
    let c = codes(&check(d.path(), &contract()).unwrap());
    assert!(
        c.contains(&"task.no_plan") && c.contains(&"task.no_change_entry"),
        "{c:?}"
    );
}

#[test]
fn valid_task_passes() {
    let d = repo_on("tempa/20261106-x");
    write(d.path(), "src/a.rs", "fn a() {}\n");
    add_plan_and_change(d.path(), "20261106-x");
    commit(d.path(), "task");
    let f = check(d.path(), &contract()).unwrap();
    assert!(errors(&f).is_empty(), "{f:?}");
}

#[test]
fn non_tempa_branch_uses_changed_plan() {
    let d = repo_on("feature/foo");
    write(d.path(), "src/a.rs", "fn a() {}\n");
    commit(d.path(), "code");
    assert!(codes(&check(d.path(), &contract()).unwrap()).contains(&"task.no_plan"));
    add_plan_and_change(d.path(), "20261106-foo");
    add_plan_and_change(d.path(), "20261106-bar");
    commit(d.path(), "two plans");
    assert!(codes(&check(d.path(), &contract()).unwrap()).contains(&"task.multiple_plans"));
}

#[test]
fn new_dependency_needs_decision() {
    let d = repo_on("tempa/20261106-dep");
    write(d.path(), "web/package.json", "{}\n");
    add_plan_and_change(d.path(), "20261106-dep");
    commit(d.path(), "dep");
    let f = check(d.path(), &contract()).unwrap();
    let dec = f
        .iter()
        .find(|x| x.code == "task.decision_required")
        .expect("decision finding");
    assert!(dec.message.contains("new-dependency"));
    write(d.path(), "docs/decisions/2026-11-06-dep.md", DECISION);
    commit(d.path(), "decision");
    assert!(!codes(&check(d.path(), &contract()).unwrap()).contains(&"task.decision_required"));
}

#[test]
fn schema_path_needs_decision() {
    let d = repo_on("tempa/20261106-mig");
    let cfg = std::fs::read_to_string(d.path().join("tempa.toml"))
        .unwrap()
        .replace(
            "schema_paths          = []",
            "schema_paths          = [\"migrations/**\"]",
        );
    write(d.path(), "tempa.toml", &cfg);
    write(d.path(), "migrations/001.sql", "create table a();\n");
    add_plan_and_change(d.path(), "20261106-mig");
    commit(d.path(), "mig");
    let f = check(d.path(), &contract()).unwrap();
    assert!(
        f.iter()
            .any(|x| x.code == "task.decision_required" && x.message.contains("schema-change"))
    );
}

#[test]
fn docs_only_pr_skips_task_rules() {
    let d = repo_on("feature/docs");
    write(d.path(), "docs/guide.md", "# Guide\n");
    commit(d.path(), "docs");
    let c = codes(&check(d.path(), &contract()).unwrap());
    assert!(!c.iter().any(|x| x.starts_with("task.")), "{c:?}");
}

#[test]
fn skip_author_skips_contract() {
    let d = repo_on("renovate/dep");
    std::fs::write(d.path().join("AGENTS.md"), "rusak\n").unwrap();
    write(d.path(), "package.json", "{}\n");
    git(d.path(), &["add", "-A"]);
    git(
        d.path(),
        &["-c", "user.name=renovate[bot]", "commit", "-qm", "bump"],
    );
    let c = codes(&check(d.path(), &contract()).unwrap());
    assert!(
        !c.iter()
            .any(|x| x.starts_with("task.") || x.starts_with("contract.")),
        "{c:?}"
    );
}

#[test]
fn diff_too_large() {
    let d = repo_on("tempa/20261106-big");
    let toml = read(d.path(), "tempa.toml")
        .replace("max_diff_lines        = 800", "max_diff_lines        = 5");
    std::fs::write(d.path().join("tempa.toml"), toml).unwrap();
    git(d.path(), &["commit", "-qam", "limit"]);
    git(d.path(), &["switch", "-q", "main"]);
    git(d.path(), &["merge", "-q", "tempa/20261106-big"]);
    git(d.path(), &["switch", "-q", "tempa/20261106-big"]);
    write(d.path(), "src/big.rs", &"x\n".repeat(10));
    add_plan_and_change(d.path(), "20261106-big");
    commit(d.path(), "big");
    let f = check(d.path(), &contract()).unwrap();
    assert!(codes(&f).contains(&"task.diff_too_large"), "{f:?}");
}

#[test]
fn drift_fails_check() {
    let d = repo_on("main-copy");
    git(d.path(), &["switch", "-q", "main"]);
    std::fs::write(d.path().join("CLAUDE.md"), "diubah\n").unwrap();
    assert!(codes(&check(d.path(), &contract()).unwrap()).contains(&"drift.modified"));
}

#[test]
fn harness_stops_at_first_failure() {
    let d = tempfile::tempdir().unwrap();
    write(
        d.path(),
        "tempa.toml",
        "[project]\nname = \"x\"\n\n[harness]\na = \"exit 0\"\nb = \"exit 3\"\nc = \"touch ran_c\"\n",
    );
    let cfg = Config::load(d.path()).unwrap();
    let f = run_harness(d.path(), &cfg).unwrap();
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].code, "harness.failed");
    assert_eq!(f[0].path.as_deref(), Some("b"));
    assert!(!d.path().join("ran_c").exists());
}

#[test]
fn full_check_runs_harness_contract_only_does_not() {
    let d = repo_on("main-copy");
    git(d.path(), &["switch", "-q", "main"]);
    let toml = read(d.path(), "tempa.toml").replace(
        "# format    = \"bun run format:check\"",
        "t = \"touch ran_t\"",
    );
    std::fs::write(d.path().join("tempa.toml"), toml).unwrap();
    tempa_core::sync::sync(d.path()).unwrap();
    check(d.path(), &contract()).unwrap();
    assert!(!d.path().join("ran_t").exists());
    check(
        d.path(),
        &CheckOpts {
            contract_only: false,
            base: None,
        },
    )
    .unwrap();
    assert!(d.path().join("ran_t").exists());
}

#[test]
fn version_mismatch() {
    let d = repo_on("main-copy");
    git(d.path(), &["switch", "-q", "main"]);
    let toml = read(d.path(), "tempa.toml").replace(env!("CARGO_PKG_VERSION"), "0.0.1");
    std::fs::write(d.path().join("tempa.toml"), toml).unwrap();
    assert!(codes(&check(d.path(), &contract()).unwrap()).contains(&"version.mismatch"));
}

#[test]
fn diff_gate_ignores_lockfiles() {
    let d = repo_on("tempa/20261106-locks");
    let toml = read(d.path(), "tempa.toml")
        .replace("max_diff_lines        = 800", "max_diff_lines        = 60");
    std::fs::write(d.path().join("tempa.toml"), toml).unwrap();
    git(d.path(), &["commit", "-qam", "limit"]);
    git(d.path(), &["switch", "-q", "main"]);
    git(d.path(), &["merge", "-q", "tempa/20261106-locks"]);
    git(d.path(), &["switch", "-q", "tempa/20261106-locks"]);
    write(d.path(), "app/bun.lock", &"x\n".repeat(200));
    write(d.path(), "app/src-tauri/Cargo.lock", &"x\n".repeat(200));
    write(d.path(), "web/package-lock.json", &"x\n".repeat(200));
    write(d.path(), "src/x.rs", "x\n");
    add_plan_and_change(d.path(), "20261106-locks");
    commit(d.path(), "locks");
    let f = check(d.path(), &contract()).unwrap();
    assert!(!codes(&f).contains(&"task.diff_too_large"), "{f:?}");

    write(d.path(), "src/big.rs", &"x\n".repeat(100));
    commit(d.path(), "big");
    let f = check(d.path(), &contract()).unwrap();
    assert!(codes(&f).contains(&"task.diff_too_large"), "{f:?}");
}

#[test]
fn is_protected_matches_config_globs() {
    let mut cfg =
        tempa_core::config::Config::parse(&tempa_core::config::default_toml("x")).unwrap();
    cfg.gates.protected_paths = vec![
        ".env*".into(),
        ".github/workflows/**".into(),
        "tempa.toml".into(),
    ];
    let p = |rel| tempa_core::check::is_protected(&cfg, rel);
    assert!(p(".env") && p(".env.local") && p("tempa.toml") && p(".github/workflows/ci.yml"));
    assert!(!p("src/env.rs") && !p("docs/tempa.toml.md"));
    cfg.gates.protected_paths = vec!["[".into()]; // invalid glob fails closed
    assert!(tempa_core::check::is_protected(&cfg, "src/a.rs"));
}
