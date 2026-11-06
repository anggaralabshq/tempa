use tempa_core::config::{Config, add_harness, default_toml};

fn write_default(dir: &std::path::Path, name: &str) {
    std::fs::write(dir.join("tempa.toml"), default_toml(name)).unwrap();
}

#[test]
fn default_toml_roundtrips() {
    let dir = tempfile::tempdir().unwrap();
    write_default(dir.path(), "billing");
    let c = Config::load(dir.path()).unwrap();
    assert_eq!(c.project.name, "billing");
    assert_eq!(c.project.tempa_version.as_deref(), Some(env!("CARGO_PKG_VERSION")));
    assert_eq!(c.docs.plan, "docs/plans/");
    assert_eq!(c.gates.max_diff_lines, 800);
    assert_eq!(c.gates.protected_paths, vec![".env*", "bitbucket-pipelines.yml", "tempa.toml"]);
    assert_eq!(c.agents.enabled.len(), 5);
    assert_eq!(c.headings.context[3], "Area terlarang");
    assert_eq!(c.ci.base, "main");
    assert_eq!(c.ci.skip_authors, vec!["renovate[bot]"]);
    assert!(c.harness.is_empty());
}

#[test]
fn harness_keeps_file_order() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("tempa.toml"),
        "[project]\nname = \"x\"\n\n[harness]\nz = \"1\"\na = \"2\"\n",
    )
    .unwrap();
    let c = Config::load(dir.path()).unwrap();
    assert_eq!(
        c.harness,
        vec![("z".to_string(), "1".to_string()), ("a".to_string(), "2".to_string())]
    );
}

#[test]
fn missing_config_mentions_init() {
    let dir = tempfile::tempdir().unwrap();
    let e = Config::load(dir.path()).unwrap_err();
    assert!(e.0.contains("tempa init"), "{}", e.0);
}

#[test]
fn add_harness_is_idempotent_and_keeps_comments() {
    let dir = tempfile::tempdir().unwrap();
    write_default(dir.path(), "billing");
    assert!(add_harness(dir.path(), "format", "cargo fmt --check").unwrap());
    assert!(!add_harness(dir.path(), "format", "cargo fmt --check").unwrap());
    let text = std::fs::read_to_string(dir.path().join("tempa.toml")).unwrap();
    assert!(text.contains("# lint = "), "{text}");
    let c = Config::load(dir.path()).unwrap();
    assert_eq!(c.harness, vec![("format".to_string(), "cargo fmt --check".to_string())]);
}

#[test]
fn add_protected_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    write_default(dir.path(), "billing");
    assert!(!tempa_core::config::add_protected(dir.path(), ".env*").unwrap());
    assert!(tempa_core::config::add_protected(dir.path(), "secrets/**").unwrap());
    let c = Config::load(dir.path()).unwrap();
    assert_eq!(c.gates.protected_paths.last().unwrap(), "secrets/**");
}
