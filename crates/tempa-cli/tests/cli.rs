use std::path::Path;
use std::process::{Command, Output};

fn tempa(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tempa"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string() + &String::from_utf8_lossy(&o.stderr)
}

fn git(dir: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8(o.stdout).unwrap()
}

fn write(dir: &Path, rel: &str, text: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn canon_repo() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    write(p, "tempa.toml", &tempa_core::config::default_toml("x"));
    write(p, "AGENTS.md", "# A\n");
    write(
        p,
        ".tempa/commands/review.md",
        "---\ndescription: R\n---\nreview\n",
    );
    git(p, &["init", "-q", "-b", "main"]);
    git(p, &["config", "user.name", "test"]);
    git(p, &["config", "user.email", "test@x"]);
    d
}

#[test]
fn sync_check_exit_codes() {
    let d = canon_repo();
    assert_eq!(tempa(d.path(), &["sync"]).status.code(), Some(0));
    assert_eq!(tempa(d.path(), &["sync", "--check"]).status.code(), Some(0));
    let p = d.path().join("CLAUDE.md");
    std::fs::write(&p, std::fs::read_to_string(&p).unwrap() + "x\n").unwrap();
    let o = tempa(d.path(), &["sync", "--check"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        stdout(&o).contains("error[drift.modified] CLAUDE.md"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn sync_fix_stages_outputs() {
    let d = canon_repo();
    tempa(d.path(), &["sync"]);
    git(d.path(), &["add", "-A"]);
    git(d.path(), &["commit", "-qm", "init"]);
    write(
        d.path(),
        ".tempa/commands/review.md",
        "---\ndescription: R\n---\nreview v2\n",
    );
    git(d.path(), &["add", "-A"]);
    let o = tempa(d.path(), &["sync", "--fix"]);
    assert_eq!(o.status.code(), Some(0), "{}", stdout(&o));
    assert!(
        git(d.path(), &["diff", "--cached", "--name-only"]).contains(".claude/commands/review.md")
    );
}

#[test]
fn sync_fix_blocks_direct_edit_with_import_hint() {
    let d = canon_repo();
    tempa(d.path(), &["sync"]);
    git(d.path(), &["add", "-A"]);
    write(d.path(), ".claude/commands/baru.md", "x\n");
    let o = tempa(d.path(), &["sync", "--fix"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stdout(&o).contains("tempa import"), "{}", stdout(&o));
}

#[test]
fn missing_config_exit_2() {
    let d = tempfile::tempdir().unwrap();
    let o = tempa(d.path(), &["sync"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(stdout(&o).contains("tempa init"));
}

#[test]
fn import_dry_run_then_yes() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "CLAUDE.md", "# Proyek\n\n## Stack\n\nGo.\n");
    write(
        d.path(),
        ".claude/commands/review.md",
        "---\ndescription: R\n---\nreview\n",
    );
    let o = tempa(d.path(), &["import"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(stdout(&o).contains("--yes"), "{}", stdout(&o));
    assert!(!d.path().join("AGENTS.md").exists());
    let o = tempa(d.path(), &["import", "--yes"]);
    assert_eq!(o.status.code(), Some(0), "{}", stdout(&o));
    assert!(stdout(&o).contains("synced CLAUDE.md"), "{}", stdout(&o));
    assert!(
        std::fs::read_to_string(d.path().join("CLAUDE.md"))
            .unwrap()
            .contains("@AGENTS.md")
    );
    assert_eq!(tempa(d.path(), &["sync", "--check"]).status.code(), Some(0));
}

#[test]
fn check_base_unavailable_is_error_in_pipelines() {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q", "-b", "work"]);
    write(
        d.path(),
        "tempa.toml",
        &tempa_core::config::default_toml("x"),
    );
    let local = Command::new(env!("CARGO_BIN_EXE_tempa"))
        .current_dir(d.path())
        .args(["check", "--contract-only"])
        .env_remove("BITBUCKET_PR_ID")
        .output()
        .unwrap();
    assert!(
        stdout(&local).contains("warning[git.base_unavailable]"),
        "{}",
        stdout(&local)
    );
    let ci = Command::new(env!("CARGO_BIN_EXE_tempa"))
        .current_dir(d.path())
        .args(["check", "--contract-only"])
        .env("BITBUCKET_PR_ID", "1")
        .output()
        .unwrap();
    assert!(
        stdout(&ci).contains("error[git.base_unavailable]"),
        "{}",
        stdout(&ci)
    );
    assert_eq!(ci.status.code(), Some(1));
}

#[test]
fn workflow_commands() {
    let d = canon_repo();
    tempa(d.path(), &["sync"]);
    git(d.path(), &["add", "-A"]);
    git(d.path(), &["commit", "-qm", "init"]);
    let o = tempa(d.path(), &["task", "new", "add-invoice"]);
    assert_eq!(o.status.code(), Some(0), "{}", stdout(&o));
    assert!(git(d.path(), &["branch", "--show-current"]).starts_with("tempa/"));
    assert_eq!(tempa(d.path(), &["task", "approve"]).status.code(), Some(0));
    let o = tempa(d.path(), &["decide", "invoice-numbering"]);
    assert!(stdout(&o).contains("docs/decisions/"), "{}", stdout(&o));
    let o = tempa(
        d.path(),
        &[
            "log",
            "--summary",
            "s",
            "--reason",
            "r",
            "--agent",
            "codex / gpt-5",
        ],
    );
    assert_eq!(o.status.code(), Some(0), "{}", stdout(&o));
    assert!(
        std::fs::read_to_string(d.path().join("CHANGES.md"))
            .unwrap()
            .contains("- Harness: pending")
    );
    let o = tempa(
        d.path(),
        &[
            "log",
            "--summary",
            "s",
            "--reason",
            "r",
            "--agent",
            "a",
            "--harness",
            "ok",
        ],
    );
    assert_eq!(o.status.code(), Some(2));
    let o = tempa(d.path(), &["pr-body"]);
    assert!(
        stdout(&o).contains("## Agent + model\n\ncodex / gpt-5"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn init_prints_created_and_devops_handoff() {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q", "-b", "main"]);
    let o = tempa(d.path(), &["init"]);
    assert_eq!(o.status.code(), Some(0), "{}", stdout(&o));
    let out = stdout(&o);
    assert!(
        out.contains("tempa.toml") && out.contains(".tempa/hooks/pre-commit"),
        "{out}"
    );
    assert!(
        out.contains("Hand docs/tempa-devops.md to whoever owns CI"),
        "{out}"
    );
    assert_eq!(
        tempa(d.path(), &["sync", "--check"]).status.code(),
        Some(0),
        "init juga sync"
    );
    assert!(d.path().join("CLAUDE.md").is_file());
}

#[test]
fn doctor_prints_table_and_exits_0_even_with_drift() {
    let d = canon_repo();
    tempa(d.path(), &["sync"]);
    write(d.path(), "CLAUDE.md", "edited\n");
    let o = tempa(d.path(), &["doctor"]);
    assert_eq!(o.status.code(), Some(0), "{}", stdout(&o));
    let out = stdout(&o);
    assert!(out.contains("agent") && out.contains("lifecycle"), "{out}");
    assert!(
        out.contains(
            "- codex commands: review.md (served via tempa_context; enforced by tempa check)"
        ),
        "{out}"
    );
    assert!(out.contains("run tempa init"), "{out}");
    assert!(out.contains("drift: 1"), "{out}");
    // Columns line up whatever the longest protected value is.
    let col = out.lines().next().unwrap().find("lifecycle").unwrap();
    for row in out.lines().skip(1).take_while(|l| !l.starts_with('-')) {
        assert_eq!(
            row.len() - row.split_whitespace().last().unwrap().len(),
            col,
            "{out}"
        );
    }
}

#[test]
fn init_refuses_claude_repo_and_keeps_claude_md() {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q", "-b", "main"]);
    write(d.path(), "CLAUDE.md", "# Billing\n\nRust.\n");
    let o = tempa(d.path(), &["init"]);
    assert_eq!(o.status.code(), Some(2), "{}", stdout(&o));
    assert!(stdout(&o).contains("tempa import --yes"), "{}", stdout(&o));
    assert_eq!(
        std::fs::read_to_string(d.path().join("CLAUDE.md")).unwrap(),
        "# Billing\n\nRust.\n"
    );
    assert!(!d.path().join("tempa.toml").exists());
}

#[test]
fn sync_fix_stages_lock() {
    let d = canon_repo();
    tempa(d.path(), &["sync"]);
    git(d.path(), &["add", "-A"]);
    git(d.path(), &["commit", "-qm", "init"]);
    write(
        d.path(),
        ".tempa/commands/review.md",
        "---\ndescription: R\n---\nreview v2\n",
    );
    git(d.path(), &["add", "-A"]);
    let o = tempa(d.path(), &["sync", "--fix"]);
    assert_eq!(o.status.code(), Some(0), "{}", stdout(&o));
    assert!(git(d.path(), &["diff", "--cached", "--name-only"]).contains(".tempa/lock.json"));
    let status = git(d.path(), &["status", "--porcelain"]);
    assert!(
        status.lines().all(|l| l.as_bytes()[1] == b' '),
        "ada yang belum di-stage:\n{status}"
    );
}

#[test]
fn mcp_stdio_roundtrip() {
    use std::io::{BufRead, BufReader, Write};
    use std::process::Stdio;
    let d = canon_repo();
    git(d.path(), &["add", "-A"]);
    git(d.path(), &["commit", "-qm", "init", "--no-verify"]);
    let sub = d.path().join("src");
    std::fs::create_dir_all(&sub).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_tempa"))
        .current_dir(&sub)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();

    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-06-18","capabilities":{{}},"clientInfo":{{"name":"t","version":"0"}}}}}}"#).unwrap();
    out.read_line(&mut line).unwrap();
    let v: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(v["result"]["protocolVersion"], "2025-06-18");

    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#
    )
    .unwrap();
    writeln!(stdin, "not json").unwrap();
    line.clear();
    out.read_line(&mut line).unwrap();
    let v: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(v["error"]["code"], -32700);

    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"tempa_status","arguments":{{}}}}}}"#).unwrap();
    line.clear();
    out.read_line(&mut line).unwrap();
    let v: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(v["id"], 2);
    assert_eq!(v["result"]["isError"], false, "{v}");
    let status: serde_json::Value =
        serde_json::from_str(v["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(
        status["branch"], "main",
        "root must be the repo, not the subdir"
    );

    drop(stdin);
    assert!(child.wait().unwrap().success());
    let mut rest = String::new();
    std::io::Read::read_to_string(&mut out, &mut rest).unwrap();
    assert!(rest.is_empty(), "nothing else on stdout: {rest}");
}
