//! Regression tests for the final-review findings (C1, C2, I1–I7).
mod common;
use common::{fixture_repo, git, read, valid_repo, write};
use tempa_core::import::import;
use tempa_core::sync::{Drift, drift, drift_findings, sync, sync_fix, sync_force};

fn commit_all(p: &std::path::Path, msg: &str) {
    git(p, &["add", "-A"]);
    git(p, &["commit", "-qm", msg, "--no-verify"]);
}

// ---- C1: sync never clobbers files it does not own ----

#[test]
fn sync_refuses_unowned_file_and_keeps_it() {
    let d = tempfile::tempdir().unwrap();
    write(
        d.path(),
        "tempa.toml",
        &tempa_core::config::default_toml("x"),
    );
    write(d.path(), "AGENTS.md", "# A\n");
    write(d.path(), "CLAUDE.md", "# Tulisan tangan\n");
    let e = sync(d.path()).unwrap_err().0;
    assert!(e.contains("CLAUDE.md") && e.contains("tempa import"), "{e}");
    assert_eq!(read(d.path(), "CLAUDE.md"), "# Tulisan tangan\n");
}

#[test]
fn sync_refuses_modified_target_but_force_overwrites() {
    let d = valid_repo();
    write(d.path(), "CLAUDE.md", "edit langsung\n");
    assert!(sync(d.path()).is_err());
    assert_eq!(read(d.path(), "CLAUDE.md"), "edit langsung\n");
    sync_force(d.path()).unwrap();
    assert!(read(d.path(), "CLAUDE.md").contains("@AGENTS.md"));
    assert!(drift(d.path()).unwrap().is_empty());
}

#[test]
fn import_keeps_keys_of_existing_agent_configs() {
    let d = fixture_repo("claude-heavy");
    write(
        d.path(),
        "opencode.json",
        r#"{"$schema":"https://opencode.ai/config.json","permission":{"edit":"ask"}}"#,
    );
    write(
        d.path(),
        ".gemini/settings.json",
        r#"{"theme":"GitHub","tools":{"sandbox":true}}"#,
    );
    write(
        d.path(),
        ".codex/config.toml",
        "model = \"o3\"\napproval_policy = \"on-request\"\n",
    );
    import(d.path(), true).unwrap();
    let oc = read(d.path(), "opencode.json");
    assert!(
        oc.contains("\"*\": \"ask\"") && oc.contains("$schema") && oc.contains("\"mcp\""),
        "{oc}"
    );
    let gm = read(d.path(), ".gemini/settings.json");
    assert!(
        gm.contains("\"theme\": \"GitHub\"") && gm.contains("fileName"),
        "{gm}"
    );
    let cx = read(d.path(), ".codex/config.toml");
    assert!(
        cx.contains("model = \"o3\"") && cx.contains("[mcp_servers.pg]"),
        "{cx}"
    );
    assert!(
        drift(d.path()).unwrap().is_empty(),
        "{:?}",
        drift(d.path()).unwrap()
    );
}

#[test]
fn import_apply_syncs_absorbed_targets() {
    let d = fixture_repo("claude-heavy");
    let r = import(d.path(), true).unwrap();
    assert!(
        r.synced.contains(&"CLAUDE.md".to_string()),
        "{:?}",
        r.synced
    );
    assert!(read(d.path(), "CLAUDE.md").contains("@AGENTS.md"));
    assert!(read(d.path(), "AGENTS.md").contains("Rust + Postgres."));
    assert!(drift(d.path()).unwrap().is_empty());
}

// ---- C2: hook stages the lock, never syncs unstaged source ----

#[test]
fn sync_fix_reports_lock_as_changed() {
    let d = valid_repo();
    write(
        d.path(),
        ".tempa/commands/r.md",
        "---\ndescription: R\n---\nr\n",
    );
    git(d.path(), &["add", "-A"]);
    let (changed, blockers) = sync_fix(d.path()).unwrap();
    assert!(blockers.is_empty(), "{blockers:?}");
    assert!(
        changed.contains(&".tempa/lock.json".to_string()),
        "{changed:?}"
    );
}

#[test]
fn sync_fix_blocks_unstaged_source_changes() {
    let d = valid_repo();
    let cmd = ".tempa/commands/review.md";
    write(d.path(), cmd, "Review.\n");
    tempa_core::sync::sync(d.path()).unwrap();
    git(d.path(), &["add", "-A"]);
    git(d.path(), &["commit", "-qm", "command", "--no-verify"]);
    write(d.path(), cmd, "Review.\nSECRET WIP NOTE\n");
    let (changed, blockers) = sync_fix(d.path()).unwrap();
    assert!(changed.is_empty());
    assert!(
        blockers
            .iter()
            .any(|f| f.code == "sync.unstaged_source" && f.path.as_deref() == Some(cmd)),
        "{blockers:?}"
    );
    assert!(!read(d.path(), ".claude/commands/review.md").contains("SECRET WIP NOTE"));
}

#[test]
fn import_never_absorbs_target_that_equals_current_render() {
    // Dev B pulled A's outputs with a stale lock: disk == render, disk != lock.
    let d = valid_repo();
    write(
        d.path(),
        ".tempa/commands/review.md",
        "---\ndescription: R\n---\nversi A\n",
    );
    sync(d.path()).unwrap();
    let target = tempa_core::sync::hash(read(d.path(), ".claude/commands/review.md").as_bytes());
    let lock = read(d.path(), ".tempa/lock.json").replace(&target, "0000");
    write(d.path(), ".tempa/lock.json", &lock);
    let r = import(d.path(), false).unwrap();
    assert!(
        !r.written.contains(&".tempa/commands/review.md".to_string()),
        "{:?}",
        r.written
    );
}

#[test]
fn import_never_overwrites_uncommitted_source() {
    let d = valid_repo();
    write(
        d.path(),
        ".tempa/commands/review.md",
        "---\ndescription: R\n---\nv1\n",
    );
    sync(d.path()).unwrap();
    commit_all(d.path(), "v1");
    write(
        d.path(),
        ".tempa/commands/review.md",
        "---\ndescription: R\n---\nv1 + WIP sumber\n",
    );
    write(
        d.path(),
        ".claude/commands/review.md",
        "edit langsung di target\n",
    );
    let r = import(d.path(), true).unwrap();
    assert!(read(d.path(), ".tempa/commands/review.md").contains("WIP sumber"));
    assert!(
        r.gaps
            .iter()
            .any(|g| g.contains(".tempa/commands/review.md")),
        "{:?}",
        r.gaps
    );
}

// ---- I1: re-import is stable ----

#[test]
fn reimport_does_not_resurrect_or_duplicate() {
    let d = fixture_repo("claude-heavy");
    import(d.path(), true).unwrap();
    write(
        d.path(),
        ".tempa/targets/claude.md",
        "Hanya untuk Claude.\n",
    );
    let agents = read(d.path(), "AGENTS.md").replace("Rust + Postgres.", "Rust + SQLite.");
    write(d.path(), "AGENTS.md", &agents);
    sync(d.path()).unwrap();
    import(d.path(), true).unwrap();
    let after = read(d.path(), "AGENTS.md");
    assert_eq!(after, agents, "re-import mengubah AGENTS.md");
    assert!(!after.contains("Hanya untuk Claude."));
}

#[test]
fn import_absorbs_only_new_paragraphs_of_edited_claude_md() {
    let d = fixture_repo("claude-heavy");
    import(d.path(), true).unwrap();
    write(
        d.path(),
        ".tempa/targets/claude.md",
        "Hanya untuk Claude.\n",
    );
    sync(d.path()).unwrap();
    let claude = read(d.path(), "CLAUDE.md") + "\nAturan baru dari Claude.\n";
    write(d.path(), "CLAUDE.md", &claude);
    import(d.path(), true).unwrap();
    let a = read(d.path(), "AGENTS.md");
    assert_eq!(a.matches("Aturan baru dari Claude.").count(), 1, "{a}");
    assert!(!a.contains("Hanya untuk Claude."), "{a}");
    assert!(drift(d.path()).unwrap().is_empty());
}

// ---- I2: drift advice is always satisfiable ----

#[test]
fn junk_and_ignored_files_are_not_drift() {
    let d = valid_repo();
    write(d.path(), ".claude/commands/.DS_Store", "x");
    write(d.path(), ".gitignore", "*.bak\n");
    write(d.path(), ".claude/commands/old.md.bak", "x");
    assert_eq!(drift(d.path()).unwrap(), vec![]);
    import(d.path(), true).unwrap();
    assert!(!d.path().join(".tempa/commands/.DS_Store").exists());
}

#[test]
fn import_absorbs_new_files_from_non_claude_dirs() {
    let d = valid_repo();
    write(
        d.path(),
        ".agents/skills/deploy/SKILL.md",
        "---\nname: deploy\ndescription: D\n---\nlangkah\n",
    );
    write(
        d.path(),
        ".opencode/commands/oc.md",
        "---\ndescription: O\n---\noc\n",
    );
    assert_eq!(drift(d.path()).unwrap().len(), 2);
    import(d.path(), true).unwrap();
    assert!(d.path().join(".tempa/skills/deploy/SKILL.md").is_file());
    assert!(d.path().join(".tempa/commands/oc.md").is_file());
    assert!(
        drift(d.path()).unwrap().is_empty(),
        "{:?}",
        drift(d.path()).unwrap()
    );
}

#[test]
fn unabsorbable_drift_names_the_source() {
    let d = valid_repo();
    write(d.path(), ".gemini/commands/g.toml", "prompt = \"x\"\n");
    let f = drift_findings(&drift(d.path()).unwrap());
    assert_eq!(f.len(), 1);
    assert!(f[0].message.contains(".tempa/commands"), "{}", f[0].message);
    assert!(!f[0].message.contains("tempa import"), "{}", f[0].message);
}

// ---- I4: MCP env + headers per agent ----

#[test]
fn mcp_env_interpolation_translated() {
    let d = fixture_repo("claude-heavy");
    import(d.path(), true).unwrap();
    let oc = read(d.path(), "opencode.json");
    assert!(oc.contains("\"DB\": \"{env:DB_URL}\""), "{oc}");
    let cx = read(d.path(), ".codex/config.toml");
    assert!(!cx.contains("${DB_URL}"), "{cx}");
    assert!(read(d.path(), ".mcp.json").contains("${DB_URL}"));
}

#[test]
fn mcp_headers_kept_and_unknown_fields_reported() {
    let d = tempfile::tempdir().unwrap();
    write(
        d.path(),
        ".mcp.json",
        r#"{"mcpServers":{"api":{"type":"http","url":"https://x/mcp","headers":{"Authorization":"Bearer ${TOKEN}"}},"local":{"command":"srv","cwd":"tools"}}}"#,
    );
    let r = import(d.path(), true).unwrap();
    assert!(read(d.path(), ".mcp.json").contains("Authorization"));
    assert!(read(d.path(), "opencode.json").contains("{env:TOKEN}"));
    assert!(read(d.path(), ".codex/config.toml").contains("bearer_token_env_var = \"TOKEN\""));
    assert!(r.gaps.iter().any(|g| g.contains("cwd")), "{:?}", r.gaps);
}

// ---- I5: generated YAML frontmatter is valid ----

#[test]
fn generated_frontmatter_quotes_unsafe_values_and_folds_blocks() {
    let d = valid_repo();
    write(
        d.path(),
        ".tempa/agents/rev.md",
        "---\ndescription: \"Reviewer: cek keamanan\"\n---\nbody\n",
    );
    write(
        d.path(),
        ".tempa/agents/fold.md",
        "---\ndescription: >\n  baris satu\n  baris dua\n---\nbody\n",
    );
    sync(d.path()).unwrap();
    let oc = read(d.path(), ".opencode/agents/rev.md");
    assert!(
        oc.contains("description: \"Reviewer: cek keamanan\""),
        "{oc}"
    );
    let fold = read(d.path(), ".gemini/agents/fold.md");
    assert!(fold.contains("description: baris satu baris dua"), "{fold}");
}

// ---- I6: symlinks ----

#[cfg(unix)]
#[test]
fn symlinked_skill_dir_inside_repo_is_imported() {
    let d = valid_repo();
    write(
        d.path(),
        "shared/adr/SKILL.md",
        "---\nname: adr\ndescription: A\n---\nadr\n",
    );
    std::fs::create_dir_all(d.path().join(".claude/skills")).unwrap();
    std::os::unix::fs::symlink("../../shared/adr", d.path().join(".claude/skills/adr")).unwrap();
    import(d.path(), true).unwrap();
    assert!(d.path().join(".tempa/skills/adr/SKILL.md").is_file());
}

#[cfg(unix)]
#[test]
fn symlink_outside_repo_is_not_copied() {
    let d = valid_repo();
    let outside = tempfile::tempdir().unwrap();
    write(outside.path(), "secret.env", "TOKEN=abc\n");
    std::fs::create_dir_all(d.path().join(".claude/commands")).unwrap();
    std::os::unix::fs::symlink(
        outside.path().join("secret.env"),
        d.path().join(".claude/commands/notes.md"),
    )
    .unwrap();
    let r = import(d.path(), true).unwrap();
    assert!(!d.path().join(".tempa/commands/notes.md").exists());
    assert!(
        r.gaps
            .iter()
            .any(|g| g.contains("notes.md") && g.contains("outside the repo")),
        "{:?}",
        r.gaps
    );
}

// ---- I7: broken base / lock is an error, not silence ----

#[test]
fn invalid_base_json_is_an_error() {
    let d = valid_repo();
    write(
        d.path(),
        ".tempa/targets/.claude/settings.json",
        "{\"hooks\":{},}\n",
    );
    let e = sync(d.path()).unwrap_err().0;
    assert!(e.contains(".tempa/targets/.claude/settings.json"), "{e}");
}

#[test]
fn conflicted_lock_is_an_error() {
    let d = valid_repo();
    write(
        d.path(),
        ".tempa/lock.json",
        "<<<<<<< HEAD\n{}\n=======\n{}\n>>>>>>> x\n",
    );
    let e = drift(d.path()).unwrap_err().0;
    assert!(e.contains(".tempa/lock.json"), "{e}");
}

#[allow(dead_code)]
fn _uses(_: Drift) {}

// ---- I3 + hook PATH guard ----

#[test]
fn init_keeps_existing_hooks_path() {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q"]);
    git(d.path(), &["config", "core.hooksPath", ".husky"]);
    tempa_core::init::init(d.path()).unwrap();
    assert_eq!(
        git(d.path(), &["config", "--get", "core.hooksPath"]).trim(),
        ".husky"
    );
}

#[test]
fn hooks_fail_clearly_without_tempa_on_path() {
    let d = tempfile::tempdir().unwrap();
    tempa_core::init::init(d.path()).unwrap();
    let o = std::process::Command::new("sh")
        .arg(d.path().join(".tempa/hooks/pre-commit"))
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("tempa: command not found"));
}

#[test]
fn init_refuses_repo_with_agent_context() {
    let d = fixture_repo("claude-heavy");
    assert_eq!(
        tempa_core::init::existing_agent_context(d.path()),
        Some("CLAUDE.md")
    );
}

// m4 review: servers in an existing .github/mcp.json are absorbed, not dropped on sync.
#[test]
fn import_absorbs_copilot_mcp_servers() {
    let d = valid_repo();
    let p = d.path();
    write(
        p,
        ".github/mcp.json",
        r#"{"mcpServers":{"gh":{"type":"http","url":"https://api.example/mcp","tools":["*"]}}}"#,
    );
    let r = tempa_core::import::import(p, true).unwrap();
    assert!(r.gaps.iter().all(|g| !g.contains("gh")), "{:?}", r.gaps);
    assert!(read(p, ".tempa/mcp.toml").contains("gh"));
    let v: serde_json::Value = serde_json::from_str(&read(p, ".github/mcp.json")).unwrap();
    assert_eq!(v["mcpServers"]["gh"]["url"], "https://api.example/mcp");
    let v: serde_json::Value = serde_json::from_str(&read(p, ".mcp.json")).unwrap();
    assert_eq!(v["mcpServers"]["gh"]["url"], "https://api.example/mcp");
}

// m4 review: Cursor's ${env:X} round-trips; its built-in variables are not env vars.
#[test]
fn cursor_mcp_variables_round_trip() {
    let d = valid_repo();
    let p = d.path();
    write(
        p,
        ".cursor/mcp.json",
        r#"{"mcpServers":{"pg":{"command":"npx","args":["${workspaceFolder}/db"],"env":{"DB":"${env:DB_URL}"}}}}"#,
    );
    tempa_core::import::import(p, true).unwrap();
    let cursor: serde_json::Value = serde_json::from_str(&read(p, ".cursor/mcp.json")).unwrap();
    assert_eq!(cursor["mcpServers"]["pg"]["env"]["DB"], "${env:DB_URL}");
    assert_eq!(
        cursor["mcpServers"]["pg"]["args"][0],
        "${workspaceFolder}/db"
    );
    let claude: serde_json::Value = serde_json::from_str(&read(p, ".mcp.json")).unwrap();
    assert_eq!(claude["mcpServers"]["pg"]["env"]["DB"], "${DB_URL}");
    let opencode: serde_json::Value = serde_json::from_str(&read(p, "opencode.json")).unwrap();
    assert_eq!(opencode["mcp"]["pg"]["environment"]["DB"], "{env:DB_URL}");
}

// m4 review: hand-written files in the new tempa-owned dirs are absorbed, not stuck as drift.
#[test]
fn import_absorbs_copilot_and_codex_agent_dirs() {
    let d = valid_repo();
    let p = d.path();
    write(
        p,
        ".github/agents/rev.agent.md",
        "---\nname: rev\ndescription: Reviews\n---\nReview it.\n",
    );
    write(
        p,
        ".codex/agents/sec.toml",
        "name = \"sec\"\ndescription = \"Security: audits\"\ndeveloper_instructions = \"\"\"\nAudit it.\n\"\"\"\n",
    );
    let r = tempa_core::import::import(p, true).unwrap();
    assert!(r.gaps.is_empty(), "{:?}", r.gaps);
    assert!(read(p, ".tempa/agents/rev.md").contains("Review it."));
    let sec = read(p, ".tempa/agents/sec.md");
    assert_eq!(
        sec,
        "---\nname: sec\ndescription: \"Security: audits\"\n---\nAudit it.\n"
    );
    assert!(drift(p).unwrap().is_empty(), "{:?}", drift(p).unwrap());
}

// A skill in .github/skills is not absorbed (its target is .agents/skills); the
// finding says where it goes.
#[test]
fn copilot_skill_dir_drift_names_the_skill_source() {
    let d = valid_repo();
    let p = d.path();
    sync(p).unwrap();
    write(p, ".github/skills/lint/SKILL.md", "Lint it.\n");
    let f = drift_findings(&drift(p).unwrap());
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].message.contains(".tempa/skills/"), "{}", f[0].message);
}
