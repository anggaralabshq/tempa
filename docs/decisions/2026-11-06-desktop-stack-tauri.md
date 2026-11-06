## Konteks

M2 needs a cross-platform desktop shell (macOS and Windows required, Linux compiling) with terminals and a contract panel that calls tempa-core directly, so the app and the CLI apply the same rules.

## Opsi

1. Tauri 2 + React + TypeScript + Vite + Bun, as Termul (MIT) uses, so its modules can be borrowed.
2. Electron: larger runtime, no direct call into Rust without a native addon.
3. A native GUI toolkit in Rust (egui, iced): no web terminal (xterm.js), terminal emulation would have to be written.

## Keputusan

Option 1, in app/ as a separate Cargo workspace (crate tempa-app) depending on tempa-core by path; the root workspace excludes it so the CLI, its MSRV 1.85 and the CI image never build Tauri. Frontend dependencies pinned to exact versions (React 18.3.1, Vite 8.1.0, Vitest 4.1.9, Tailwind 4.3.1, TypeScript 7.0.2, @tauri-apps/cli 2.12.2); xterm.js stable 6.0.0 when the terminal lands.

## Konsekuensi

Contributors to the app need Bun and the Tauri system dependencies; the [harness] app commands make that true for everyone working in this repo (owner's decision). Two Cargo.lock files. Termul code is copied only where small and self-contained (login PATH), with notices.

## Agent + model

claude-code / claude-opus-5-5
