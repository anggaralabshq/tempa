---
task_id: 20261106-app-scaffold
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

M2 task 2/7: scaffold the desktop app in app/ (Tauri 2, React + TypeScript, Vite, Bun, Tailwind) as a separate Cargo workspace depending on tempa-core, with an empty three-region window (Projects, Terminal, Contract).

## Langkah

1. Vitest test: App renders three regions (RED, then App).
2. app/src-tauri crate tempa-app with its own [workspace]; root workspace excludes it.
3. tauri.conf.json, capabilities, icons from app-icon.svg.
4. bun tauri dev opens the window on macOS.

## File yang disentuh

app/ (package.json, bun.lock, tsconfig, vite.config.ts, index.html, src/, src-tauri/), Cargo.toml (exclude), .gitignore.

## Risiko

Tauri build toolchain is now needed for the app harness commands (owner adds them to [harness]). @swc/core postinstall is blocked by Bun; the native binary comes from optional dependencies and the build works without it.
