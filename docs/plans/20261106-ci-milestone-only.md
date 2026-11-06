---
task_id: 20261106-ci-milestone-only
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Run the Windows CI once per milestone instead of on every task PR and every push to main, to save GitHub compute and dev time (owner's request). Task PRs target a milestone branch (m3) and are gated by local checks.

## Langkah

1. app-windows.yml: pull_request only for base main; drop the push-to-main trigger (a merged PR was already tested); add workflow_dispatch for manual runs.
2. Milestone flow: task PRs -> m3 (local gate: cargo test, clippy, bun test, tsc); one PR m3 -> main runs CI.

## File yang disentuh

.github/workflows/app-windows.yml (protected path; owner asked for this change).

## Risiko

A Windows-only break is found at milestone end instead of per task; a manual workflow_dispatch run on the milestone branch covers risky tasks.
