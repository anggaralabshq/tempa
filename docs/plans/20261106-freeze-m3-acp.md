---
task_id: 20261106-freeze-m3-acp
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Freeze M3, part 2/3 (decision docs/decisions/2026-11-06-refocus-on-contract-sync.md): remove the ACP client connection, session and its tests, unreferenced since part 1.

## Langkah

1. Delete app/src-tauri/src/acp/{conn,session,tests}.rs.
2. App tests green (28/28).

## File yang disentuh

app/src-tauri/src/acp/{conn,session,tests}.rs.

## Risiko

None: the files were no longer compiled. The code lives on branch m3.
