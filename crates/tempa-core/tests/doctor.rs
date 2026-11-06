mod common;
use common::{fixture_repo, write};
use tempa_core::config::Agent;
use tempa_core::doctor::doctor;
use tempa_core::import::import;
use tempa_core::sync::sync;

#[test]
fn doctor_after_import() {
    let d = fixture_repo("claude-heavy");
    import(d.path(), true).unwrap();
    sync(d.path()).unwrap();
    let r = doctor(d.path()).unwrap();
    let claude = r.rows.iter().find(|x| x.agent == Agent::Claude).unwrap();
    assert!(claude.gaps.is_empty(), "{:?}", claude.gaps);
    assert_eq!(claude.protected, "settings deny");
    assert!(claude.files > 0);
    let codex = r.rows.iter().find(|x| x.agent == Agent::Codex).unwrap();
    assert_eq!(codex.gaps.iter().filter(|g| g.kind == "commands").count(), 3);
    assert_eq!(codex.protected, "gap (CI)");
    assert_eq!(codex.lifecycle, "M3");
    assert!(r.legacy.contains(&".cursorrules".to_string()), "{:?}", r.legacy);
    assert!(r.hooks_ok);
    assert_eq!(r.drift, 0);
}

#[test]
fn doctor_reports_legacy_mdc_but_not_generated_and_counts_drift() {
    let d = fixture_repo("claude-heavy");
    import(d.path(), true).unwrap();
    sync(d.path()).unwrap();
    write(d.path(), ".cursor/rules/old.mdc", "x\n");
    write(d.path(), "CLAUDE.md", "edited\n");
    let r = doctor(d.path()).unwrap();
    assert!(r.legacy.contains(&".cursor/rules/old.mdc".to_string()), "{:?}", r.legacy);
    assert!(!r.legacy.iter().any(|l| l.ends_with("tempa.mdc")));
    assert!(r.drift >= 1);
}

#[test]
fn doctor_hooks_not_installed() {
    let d = fixture_repo("claude-heavy");
    write(d.path(), "tempa.toml", &tempa_core::config::default_toml("x"));
    write(d.path(), "AGENTS.md", "# A\n");
    let r = doctor(d.path()).unwrap();
    assert!(!r.hooks_ok);
}
