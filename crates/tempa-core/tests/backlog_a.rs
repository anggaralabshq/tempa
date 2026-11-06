//! Batch A: pilot findings (P1–P6) and deferred minors from the M1 final review.
mod common;
use common::{git, read, valid_repo, write};
use tempa_core::check::{CheckOpts, check, run_harness};
use tempa_core::config::{Config, default_toml};
use tempa_core::md::sections;
use tempa_core::task::{LogEntry, decide, log, pr_body, task_new};

fn entry() -> LogEntry {
    LogEntry {
        summary: "s".into(),
        reason: "r".into(),
        decision: None,
        harness: "lulus".into(),
        agent: "codex / gpt-5".into(),
    }
}

// P1 + P2 + P3: pr-body
#[test]
fn pr_body_lists_only_decision_docs_reports_harness_and_keeps_heading_levels() {
    let d = valid_repo();
    task_new(d.path(), "x").unwrap();
    let dec = decide(d.path(), "pakai-sqlite").unwrap();
    log(d.path(), &entry()).unwrap();
    git(d.path(), &["add", "-A"]);
    git(d.path(), &["commit", "-qm", "x", "--no-verify"]);
    let body = pr_body(d.path()).unwrap();
    assert!(body.contains(&format!("- {dec}")), "{body}");
    assert!(!body.contains(".gitkeep"), "{body}");
    assert!(!body.contains("Bitbucket"), "{body}");
    assert!(body.contains("lulus"), "{body}");
    let top: Vec<_> = sections(&body)
        .into_iter()
        .filter(|s| s.level == 2)
        .map(|s| s.title)
        .collect();
    assert_eq!(
        top,
        [
            "Ringkasan",
            "Link plan",
            "Decision terkait",
            "Entri changes",
            "Hasil harness",
            "Review",
            "Agent + model"
        ]
    );
}

// P4: harness output is captured; failure shows the tail
#[test]
fn harness_failure_message_contains_output_tail() {
    let d = valid_repo();
    let mut cfg = Config::load(d.path()).unwrap();
    cfg.harness = vec![("test".into(), "printf 'baris-%s' penting; exit 3".into())];
    let f = run_harness(d.path(), &cfg).unwrap();
    assert_eq!(f.len(), 1);
    assert!(
        f[0].message.contains("exit 3") && f[0].message.contains("baris-penting"),
        "{}",
        f[0].message
    );
}

// P5: defaults are platform- and stack-neutral
#[test]
fn default_config_is_platform_neutral() {
    let cfg = Config::parse(&default_toml("x")).unwrap();
    assert_eq!(cfg.gates.protected_paths, vec![".env*", "tempa.toml"]);
    assert!(cfg.gates.schema_paths.is_empty() && cfg.gates.public_api_paths.is_empty());
    assert!(
        default_toml("x").contains("bitbucket-pipelines.yml"),
        "contoh tetap ada sebagai komentar"
    );
}

// P6: no harness configured is a visible warning
#[test]
fn empty_harness_is_a_warning() {
    let d = valid_repo();
    let f = check(
        d.path(),
        &CheckOpts {
            contract_only: false,
            base: None,
        },
    )
    .unwrap();
    assert!(f.iter().any(|x| x.code == "harness.empty"), "{f:?}");
    let f = check(
        d.path(),
        &CheckOpts {
            contract_only: true,
            base: None,
        },
    )
    .unwrap();
    assert!(!f.iter().any(|x| x.code == "harness.empty"));
}

// D1: merge commits do not defeat skip_authors
#[test]
fn skip_authors_ignores_merge_commits() {
    let d = valid_repo();
    git(d.path(), &["checkout", "-qb", "deps"]);
    write(d.path(), "src/lib.rs", "x\n");
    git(d.path(), &["add", "-A"]);
    git(
        d.path(),
        &[
            "-c",
            "user.name=renovate[bot]",
            "commit",
            "-qm",
            "deps",
            "--no-verify",
        ],
    );
    git(d.path(), &["checkout", "-q", "main"]);
    write(d.path(), "other.txt", "y\n");
    git(d.path(), &["add", "-A"]);
    git(d.path(), &["commit", "-qm", "main moves", "--no-verify"]);
    git(d.path(), &["checkout", "-q", "deps"]);
    git(d.path(), &["merge", "-q", "--no-edit", "main"]); // merge commit by `test`
    let authors = tempa_core::gitx::authors(d.path(), "main").unwrap();
    assert_eq!(authors.into_iter().collect::<Vec<_>>(), ["renovate[bot]"]);
}

// D3: clear error messages
#[test]
fn task_new_outside_git_says_so() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "tempa.toml", &default_toml("x"));
    let e = task_new(d.path(), "x").unwrap_err().0;
    assert!(e.contains("bukan repo git"), "{e}");
}

#[test]
fn unreadable_config_is_not_reported_as_missing() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir(d.path().join("tempa.toml")).unwrap(); // exists, unreadable as file
    let e = Config::load(d.path()).unwrap_err().0;
    assert!(!e.contains("tidak ditemukan"), "{e}");
}

#[test]
fn default_toml_escapes_project_name() {
    let cfg = Config::parse(&default_toml("a\"b\\c")).unwrap();
    assert_eq!(cfg.project.name, "a\"b\\c");
}

// D4: Claude command syntax translated for Gemini, positional args reported
#[test]
fn gemini_commands_translate_shell_and_file_syntax() {
    let d = valid_repo();
    write(
        d.path(),
        ".tempa/commands/c.md",
        "---\ndescription: C\n---\nStatus: !`git status`\nLihat @src/main.rs dan email a@b.co\n",
    );
    write(d.path(), ".tempa/commands/p.md", "Ganti $1 jadi $2\n");
    tempa_core::sync::sync(d.path()).unwrap();
    let c = read(d.path(), ".gemini/commands/c.toml");
    assert!(
        c.contains("!{git status}") && c.contains("@{src/main.rs}") && c.contains("a@b.co"),
        "{c}"
    );
    let cfg = Config::load(d.path()).unwrap();
    let src = tempa_core::source::load(d.path(), &cfg).unwrap();
    let r = tempa_core::targets::render(tempa_core::config::Agent::Gemini, &cfg, &src);
    assert!(
        r.gaps
            .iter()
            .any(|g| g.kind == "commands" && g.name.contains("p.md")),
        "{:?}",
        r.gaps
    );
}

// D5: mutating hook commands are not harness gates
#[test]
fn mutating_hooks_are_not_harness() {
    let d = tempfile::tempdir().unwrap();
    write(
        d.path(),
        ".claude/settings.json",
        r#"{"hooks":{"PostToolUse":[{"matcher":"Edit","hooks":[{"type":"command","command":"npx eslint --fix ."},{"type":"command","command":"cargo test"}]}]}}"#,
    );
    let r = tempa_core::import::import(d.path(), false).unwrap();
    assert_eq!(
        r.harness_added,
        vec![("test".to_string(), "cargo test".to_string())]
    );
    assert!(
        r.gaps.iter().any(|g| g.contains("eslint --fix")),
        "{:?}",
        r.gaps
    );
}

// D6: markdown fences and indented headings
#[test]
fn sections_match_fence_kinds_and_indented_headings() {
    let s = sections("## A\n~~~\n```\n# bukan\n~~~\n   ## B\nisi\n");
    assert_eq!(
        s.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(),
        ["A", "B"]
    );
}
