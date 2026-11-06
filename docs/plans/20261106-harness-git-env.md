---
task_id: 20261106-harness-git-env
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Harness commands run by `tempa check` from a git hook must not inherit the hook's repository-local git variables.

## Steps

- Regression test (own binary, sets GIT_DIR/GIT_WORK_TREE/GIT_INDEX_FILE): a harness command running git in another repo sees that repo.
- `run_harness` removes the variables from `git rev-parse --local-env-vars` before spawning each command.

## Files touched

crates/tempa-core/src/check.rs, crates/tempa-core/tests/harness_env.rs

## Risks

A harness command that relied on GIT_DIR from the hook now discovers the repo from its working directory (`root`), which is the same repo in every normal setup.
