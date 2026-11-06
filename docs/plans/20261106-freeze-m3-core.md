---
task_id: 20261106-freeze-m3-core
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Freeze M3, part 3/3 (decision docs/decisions/2026-11-06-refocus-on-contract-sync.md): remove the fake ACP agent and core task::set_state/STATES, which only the M3 host used.

## Langkah

1. Delete app/src-tauri/tests/fake-agent.mjs.
2. Restore crates/tempa-core/src/task.rs and tests/task.rs to their pre-M3 state.
3. Workspace tests and clippy green; app/ and crates/ equal PR #15 again.

## File yang disentuh

app/src-tauri/tests/fake-agent.mjs, crates/tempa-core/src/task.rs, crates/tempa-core/tests/task.rs.

## Risiko

Core public API shrinks (set_state, STATES); no CLI command used them.
