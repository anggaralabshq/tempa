---
task_id: 20261106-fix-hookless-test-repos
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Fix the Windows CI failures merged in PR #16 and #17: the verify/mcp_host test repos committed through tempa's pre-commit hook, which needs the tempa CLI on PATH; the Windows runner has none.

## Langkah

1. Reproduce: run the app test binary with a PATH without tempa → 5 tests fail at `git commit`.
2. The test helper commits with --no-verify.
3. Same run passes 43/43.

## File yang disentuh

app/src-tauri/src/verify.rs.

## Risiko

None for the product; test-only. Process fix: CI status is now read from `gh pr checks` itself, not from a pipe.
