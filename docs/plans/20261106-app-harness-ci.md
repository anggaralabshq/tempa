---
task_id: 20261106-app-harness-ci
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Make the app part of the repo's gate (owner approved writing these protected files): app typecheck, Vitest, clippy and backend tests in [harness]; backend tests on Windows in GitHub Actions.

## Langkah

1. tempa.toml [harness]: app_check, app_test, app_rust.
2. .github/workflows/app-windows.yml: cargo test for app/src-tauri on windows-latest, for PRs touching app/ or tempa-core.
3. HARNESS.md gates updated.
4. First Windows run failed (3 pty tests): ConPTY waits for an answer to its cursor-position request (xterm.js answers in the app; tests now answer it), and kill left the console open. kill now closes the pty, which ends the whole process tree on both platforms.

## File yang disentuh

tempa.toml, .github/workflows/app-windows.yml, HARNESS.md, app/src-tauri/src/pty.rs.

## Risiko

Everyone running tempa check here now needs Bun and the Tauri system dependencies (owner's decision). First Windows run may reveal ConPTY differences; fixes go in follow-up PRs.
