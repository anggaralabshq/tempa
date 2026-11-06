---
task_id: 20261106-m2-review-fixes
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Fix the Important findings of the M2 final review: destructive actions list exactly what they change and refuse if it changed, missing folders cannot be recreated, no console windows on Windows, a busy folder cannot stall the watcher, no read-event loop on Linux, a big paste cannot freeze the app.

## Langkah

1. Absorb preview adds the targets the import may overwrite (core ImportReport.absorbed).
2. contract_run takes the reviewed list; Absorb/Import/Discard refuse with "review again" when a fresh preview differs.
3. Missing project folder: status reports it, panel hides actions, run refuses.
4. tempa_core::command sets CREATE_NO_WINDOW on Windows (git, tempa --version, pwsh probe); shell detection cached.
5. Watcher drops ignored paths and read events in the notify callback; bursts end after 2 s.
6. Each pty has a writer thread fed by a channel; pty_write never blocks.

## File yang disentuh

crates/tempa-core/src/{lib.rs,gitx.rs,import.rs}, app/src-tauri/src/{contract.rs,env.rs,pty.rs,watcher.rs,lib.rs}, app/src/{contract/ContractPanel.tsx,contract/ContractPanel.test.tsx,lib/ipc.ts}.

## Risiko

CREATE_NO_WINDOW is only compile-checked (Windows CI); the visual check is the Windows teammate's manual test. Eight review minors are deferred to the backlog.
