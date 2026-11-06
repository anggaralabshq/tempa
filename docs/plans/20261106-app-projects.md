---
task_id: 20261106-app-projects
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

M2 task 5/7: projects. Add a repo folder with a folder picker, keep the list across restarts, and give each project its own tabs and terminals that survive switching between projects.

## Langkah

1. projects.rs: list/add/remove on projects.json in the app config dir (atomic write; id from the canonical path; same folder twice returns the existing entry; corrupt file kept as .bak; Windows \\?\ prefix dropped).
2. Commands projects_list/add/remove; tauri-plugin-dialog for the picker.
3. Sidebar; App keeps every project's Workspace mounted (hidden when inactive); only the visible one takes shortcuts; removing a project kills its terminals.

## File yang disentuh

app/src-tauri/{Cargo.toml,Cargo.lock,src/lib.rs,src/projects.rs,capabilities/default.json}, app/{package.json,bun.lock,src/App.tsx,src/App.test.tsx,src/lib/ipc.ts,src/sidebar/Sidebar.tsx,src/layout/Workspace.tsx}.

## Risiko

Many open projects keep many shells alive (by design). A project folder deleted while the app runs: its terminals keep running until removed; Contract panel (task 6) reports it.
