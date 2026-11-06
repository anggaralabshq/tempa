---
task_id: 20261106-app-terminal
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

M2 task 3/7: a real terminal in the app. One pty runs the user's login shell; output streams to xterm.js as raw bytes; the shell gets the user's interactive-shell PATH even when the app is launched from Finder/Dock.

## Langkah

1. env.rs (adapted from Termul env_refresh.rs): merge_path_segments, login-shell PATH probe with a 5 s timeout, user_shell (SHELL / pwsh / powershell).
2. pty.rs: PtyManager spawn/write/resize/kill/kill_all on portable-pty; waiter closes the session on exit (ConPTY EOF); on_exit after the last chunk.
3. Tauri commands pty_spawn (async), pty_write, pty_resize, pty_kill; kill_all on app exit.
4. Terminal.tsx (xterm 6 + fit + web-links); unmount kills the shell.
5. THIRD_PARTY_NOTICES.md; Windows test workflow proposed to the owner.

## File yang disentuh

app/src-tauri/{Cargo.toml,Cargo.lock,src/lib.rs,src/env.rs,src/pty.rs}, app/{package.json,bun.lock,src/App.tsx,src/App.test.tsx,src/lib/ipc.ts,src/terminal/Terminal.tsx}, THIRD_PARTY_NOTICES.md.

## Risiko

Shell startup files that wait for input (probe killed after 5 s, inherited PATH used). Windows ConPTY not exercised on this machine: covered by the proposed windows-latest workflow. Windows PATH changes after login are not picked up until re-login (Termul re-reads the registry; deferred).
