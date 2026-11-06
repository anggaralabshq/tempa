---
task_id: 20261106-app-contract-panel
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

M2 task 6/7: the Contract panel for the active project: contract findings, drift with Absorb/Discard, per-agent summary, git hooks, and a warning when the tempa CLI on PATH (used by the hooks) is missing or a different version.

## Langkah

1. tempa-core init::install_hooks (hooks only; init calls it).
2. app contract.rs: status (check --contract-only split into drift and other findings, doctor rows, hooks, CLI version), preview (absorb = import dry run, discard = drift paths sync --force rewrites), run (absorb/import, discard, init, hooks).
3. ContractPanel: every action shows what it will change and runs only after Confirm; folders without tempa.toml offer Import or Init.

## File yang disentuh

crates/tempa-core/src/init.rs, crates/tempa-core/tests/init.rs, app/src-tauri/src/{contract.rs,lib.rs}, app/src/{contract/ContractPanel.tsx,contract/ContractPanel.test.tsx,lib/ipc.ts,App.tsx,App.test.tsx}.

## Risiko

Absorb/Discard act on the whole project (same as the CLI), not per file; the dialog lists every file first. Status runs check and doctor on every refresh (fast on repos of this size; watcher-driven refresh in task 7).
