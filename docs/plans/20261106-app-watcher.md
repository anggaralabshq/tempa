---
task_id: 20261106-app-watcher
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

M2 task 7/7: a watcher per project. Source edits (context file, tempa.toml, .tempa/**) sync automatically with the same no-clobber rules as `tempa sync`; direct edits to generated files are reported (sidebar badge, panel refresh) and never absorbed on their own.

## Langkah

1. app watcher.rs: classify (Source / Other / Ignore), burst debounce (300 ms quiet), watch(root, on_update) on notify; Source batch -> sync_with(no overwrite) + drift; other batch -> emit only when the drift count changes, so tempa's own writes never come back as an alert.
2. lib.rs: watch_start / watch_stop commands, `contract-changed` event.
3. App: watch every project, badge = drift + contract errors, refresh the panel on each update; removing a project stops its watcher.

## File yang disentuh

app/src-tauri/{Cargo.toml,Cargo.lock,src/watcher.rs,src/lib.rs}, app/src/{App.tsx,App.test.tsx,lib/ipc.ts,sidebar/Sidebar.tsx}, docs/decisions/.

## Risiko

Gitignored paths other than .git, target and node_modules still reach the watcher (cost: a drift recount, no alert unless drift changes). Each project keeps one recursive OS watch; very large repos may hit inotify limits on Linux (M4).
