---
task_id: 20261106-ci-portability
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

The milestone CI passes on Linux, macOS and Windows.

## Steps

- e2e step 8: the bare origin uses `-b main` (CI runners default to master, so the clone had no commit), and origin and the second clone live outside the repo under test.
- Tests whose harness command is sh syntax: a `cmd` variant (backlog_a) or `cfg(unix)` (harness_env).
- CI runs `cargo test --no-fail-fast` so every failing test binary shows in one run.

## Files touched

scripts/e2e.sh, crates/tempa-core/tests/{backlog_a,harness_env}.rs, .github/workflows/ci.yml

## Risks

Windows-only failures in other test binaries are still unknown until the next CI run.
