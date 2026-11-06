mod common;
use common::{fixture_repo, git, read, write};
use tempa_core::check::{CheckOpts, check};
use tempa_core::init::init;
use tempa_core::sync::sync;

#[test]
fn init_fresh_repo_then_check_runs() {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q"]);
    let created = init(d.path()).unwrap();
    for f in [
        "tempa.toml",
        "AGENTS.md",
        "HARNESS.md",
        "CHANGES.md",
        "docs/plans/.gitkeep",
        "docs/decisions/.gitkeep",
        ".gitattributes",
        ".tempa/hooks/pre-commit",
        ".tempa/hooks/pre-push",
        "docs/tempa-devops.md",
    ] {
        assert!(created.contains(&f.to_string()), "{f} not in {created:?}");
    }
    assert_eq!(
        git(d.path(), &["config", "core.hooksPath"]).trim(),
        ".tempa/hooks"
    );
    assert!(read(d.path(), ".gitattributes").contains("CHANGES.md merge=union"));
    assert!(read(d.path(), ".tempa/hooks/pre-commit").contains("tempa sync --fix"));
    assert!(read(d.path(), ".tempa/hooks/pre-push").contains("tempa check"));
    let devops = read(d.path(), "docs/tempa-devops.md");
    assert!(
        devops.contains(&format!("tempa:{}", env!("CARGO_PKG_VERSION"))),
        "{devops}"
    );
    assert!(devops.contains("tempa check --contract-only") && devops.contains("depth: full"));
    assert!(read(d.path(), "AGENTS.md").contains("## Area terlarang"));
    sync(d.path()).unwrap();
    let f = check(
        d.path(),
        &CheckOpts {
            contract_only: true,
            base: None,
        },
    )
    .unwrap();
    assert!(f.iter().any(|x| x.code == "git.base_unavailable"), "{f:?}");
    assert!(
        f.iter().any(|x| x.code == "contract.empty_heading"),
        "{f:?}"
    );
}

#[test]
fn init_never_overwrites() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "AGENTS.md", "x\n");
    write(d.path(), ".gitattributes", "*.png binary\n");
    let created = init(d.path()).unwrap();
    assert_eq!(read(d.path(), "AGENTS.md"), "x\n");
    assert!(!created.contains(&"AGENTS.md".to_string()));
    assert_eq!(
        read(d.path(), ".gitattributes"),
        "*.png binary\nCHANGES.md merge=union\n"
    );
}

#[test]
fn init_twice_creates_nothing() {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q"]);
    init(d.path()).unwrap();
    assert!(init(d.path()).unwrap().is_empty());
}

#[test]
fn init_outside_git_ok() {
    let d = tempfile::tempdir().unwrap();
    assert!(!init(d.path()).unwrap().is_empty());
}

#[test]
fn init_respects_existing_config_names() {
    let d = tempfile::tempdir().unwrap();
    write(
        d.path(),
        "tempa.toml",
        &tempa_core::config::default_toml("x").replace("\"CHANGES.md\"", "\"docs/CHANGELOG.md\""),
    );
    init(d.path()).unwrap();
    assert!(d.path().join("docs/CHANGELOG.md").exists());
    assert!(!d.path().join("CHANGES.md").exists());
    assert!(read(d.path(), ".gitattributes").contains("docs/CHANGELOG.md merge=union"));
}

#[cfg(unix)]
#[test]
fn hooks_executable() {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    init(d.path()).unwrap();
    let mode = std::fs::metadata(d.path().join(".tempa/hooks/pre-commit"))
        .unwrap()
        .permissions()
        .mode();
    assert_ne!(mode & 0o111, 0);
}

#[test]
fn import_on_bare_repo_runs_init() {
    let d = fixture_repo("claude-heavy");
    let r = tempa_core::import::import(d.path(), true).unwrap();
    assert!(d.path().join("HARNESS.md").exists());
    assert!(d.path().join("docs/tempa-devops.md").exists());
    assert!(r.initialized.contains(&"HARNESS.md".to_string()));
    assert!(
        read(d.path(), "AGENTS.md").starts_with("# Billing API\n"),
        "import content wins over the init template"
    );
}
