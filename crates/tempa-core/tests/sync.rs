mod common;
use common::{read, write};
use std::path::Path;
use tempa_core::config::default_toml;
use tempa_core::sync::{Drift, drift, hash, sync, sync_fix};

fn canon_repo() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    write(p, "tempa.toml", &default_toml("x"));
    write(p, "AGENTS.md", "# A\n");
    write(
        p,
        ".tempa/commands/review.md",
        "---\ndescription: R\n---\nreview\n",
    );
    d
}

fn crlf(p: &Path, rel: &str) {
    let t = read(p, rel).replace('\n', "\r\n");
    std::fs::write(p.join(rel), t).unwrap();
}

#[test]
fn hash_ignores_crlf() {
    assert_eq!(hash(b"a\r\nb\n"), hash(b"a\nb\n"));
    assert_ne!(hash(b"a"), hash(b"b"));
    assert_eq!(hash(b"").len(), 64);
}

#[test]
fn sync_twice_is_noop() {
    let d = canon_repo();
    let first = sync(d.path()).unwrap();
    assert!(first.contains(&"CLAUDE.md".to_string()));
    assert!(first.contains(&".claude/commands/review.md".to_string()));
    assert!(sync(d.path()).unwrap().is_empty());
    assert!(drift(d.path()).unwrap().is_empty());
    assert!(read(d.path(), ".tempa/lock.json").contains("\"CLAUDE.md\""));
}

#[test]
fn crlf_checkout_is_not_drift() {
    let d = canon_repo();
    sync(d.path()).unwrap();
    crlf(d.path(), "CLAUDE.md");
    crlf(d.path(), ".claude/commands/review.md");
    assert_eq!(drift(d.path()).unwrap(), vec![]);
    assert!(sync(d.path()).unwrap().is_empty());
}

#[test]
fn direct_edit_is_modified() {
    let d = canon_repo();
    sync(d.path()).unwrap();
    std::fs::write(
        d.path().join("CLAUDE.md"),
        read(d.path(), "CLAUDE.md") + "x\n",
    )
    .unwrap();
    assert_eq!(
        drift(d.path()).unwrap(),
        vec![Drift::Modified("CLAUDE.md".into())]
    );
}

#[test]
fn source_change_is_stale() {
    let d = canon_repo();
    sync(d.path()).unwrap();
    write(
        d.path(),
        ".tempa/commands/review.md",
        "---\ndescription: R\n---\nreview v2\n",
    );
    assert!(
        drift(d.path())
            .unwrap()
            .contains(&Drift::Stale(".claude/commands/review.md".into()))
    );
}

#[test]
fn deleted_output_is_missing() {
    let d = canon_repo();
    sync(d.path()).unwrap();
    std::fs::remove_file(d.path().join("CLAUDE.md")).unwrap();
    assert_eq!(
        drift(d.path()).unwrap(),
        vec![Drift::Missing("CLAUDE.md".into())]
    );
}

#[test]
fn new_file_in_managed_dir_is_untracked() {
    let d = canon_repo();
    sync(d.path()).unwrap();
    write(d.path(), ".claude/commands/baru.md", "baru\n");
    assert_eq!(
        drift(d.path()).unwrap(),
        vec![Drift::Untracked(".claude/commands/baru.md".into())]
    );
}

#[test]
fn removed_source_removes_output() {
    let d = canon_repo();
    sync(d.path()).unwrap();
    std::fs::remove_file(d.path().join(".tempa/commands/review.md")).unwrap();
    let changed = sync(d.path()).unwrap();
    assert!(changed.contains(&".claude/commands/review.md".to_string()));
    assert!(!d.path().join(".claude/commands/review.md").exists());
    assert!(!read(d.path(), ".tempa/lock.json").contains("review.md"));
}

#[test]
fn fix_applies_stale_but_blocks_direct_edit() {
    let d = canon_repo();
    sync(d.path()).unwrap();
    write(d.path(), "AGENTS.md", "# A v2\n");
    let (changed, blockers) = sync_fix(d.path()).unwrap();
    assert!(
        changed.contains(&".cursor/rules/tempa.mdc".to_string()),
        "{changed:?}"
    );
    assert!(blockers.is_empty());
    assert!(drift(d.path()).unwrap().is_empty());

    let edited = read(d.path(), "CLAUDE.md") + "x\n";
    std::fs::write(d.path().join("CLAUDE.md"), &edited).unwrap();
    let (changed, blockers) = sync_fix(d.path()).unwrap();
    assert!(changed.is_empty());
    assert_eq!(blockers.len(), 1);
    assert_eq!(
        (blockers[0].code, blockers[0].path.as_deref()),
        ("drift.modified", Some("CLAUDE.md"))
    );
    assert_eq!(read(d.path(), "CLAUDE.md"), edited);
}

#[test]
fn preexisting_handwritten_target_is_modified() {
    let d = canon_repo();
    write(d.path(), "CLAUDE.md", "tulisan tangan\n");
    assert!(
        drift(d.path())
            .unwrap()
            .contains(&Drift::Modified("CLAUDE.md".into()))
    );
}

#[test]
fn orphan_from_removed_source_is_stale_not_untracked() {
    let d = canon_repo();
    sync(d.path()).unwrap();
    std::fs::remove_file(d.path().join(".tempa/commands/review.md")).unwrap();
    let d2 = drift(d.path()).unwrap();
    assert!(
        d2.contains(&Drift::Stale(".claude/commands/review.md".into())),
        "{d2:?}"
    );
    assert!(
        !d2.iter().any(|x| matches!(x, Drift::Untracked(_))),
        "{d2:?}"
    );
    let (changed, blockers) = sync_fix(d.path()).unwrap();
    assert!(blockers.is_empty());
    assert!(changed.contains(&".claude/commands/review.md".to_string()));
}
