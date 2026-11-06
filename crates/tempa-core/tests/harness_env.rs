// Own test binary: it sets process-wide environment variables.
mod common;
use common::valid_repo;
use tempa_core::check::run_harness;
use tempa_core::config::Config;

/// Git exports GIT_DIR (and in pre-commit GIT_INDEX_FILE) to hooks, notably from a
/// worktree. A harness command that runs git in another repo, like a test suite
/// creating temp repos, must not inherit them, or it writes into the pushing repo.
#[test]
fn harness_commands_do_not_inherit_the_hooks_git_environment() {
    let hook_repo = valid_repo();
    let d = valid_repo();
    let other = tempfile::tempdir().unwrap();
    common::git_init(other.path());
    // SAFETY: the only test in this binary; nothing else reads the environment.
    unsafe {
        std::env::set_var("GIT_DIR", hook_repo.path().join(".git"));
        std::env::set_var("GIT_WORK_TREE", hook_repo.path());
        std::env::set_var("GIT_INDEX_FILE", hook_repo.path().join(".git/other-index"));
    }
    let mut cfg = Config::load(d.path()).unwrap();
    let seen = d.path().join("seen.txt");
    cfg.harness = vec![(
        "test".into(),
        format!(
            "cd '{}' && git rev-parse --show-toplevel > '{}'",
            other.path().display(),
            seen.display()
        ),
    )];
    let f = run_harness(d.path(), &cfg).unwrap();
    assert!(f.is_empty(), "{f:?}");
    let top = std::fs::read_to_string(&seen).unwrap();
    assert_eq!(
        std::fs::canonicalize(top.trim()).unwrap(),
        std::fs::canonicalize(other.path()).unwrap()
    );
}
