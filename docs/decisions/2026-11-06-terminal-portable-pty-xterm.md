## Konteks

The app needs real terminals (macOS and Windows) for agent CLIs, streaming raw bytes to a web terminal widget.

## Opsi

1. portable-pty 0.9 (wezterm; Unix pty + Windows ConPTY) with xterm.js 6.0.0 stable + fit/web-links addons.
2. Termul's PTY manager: ~5,700 lines tied to credential claims, ledgers and a web relay that tempa does not need.
3. Spawn shells on plain pipes: no line editing, no colors, agent TUIs break.

## Keputusan

Option 1. Raw output goes over a Tauri Channel as an ArrayBuffer (no JSON byte arrays); xterm decodes partial UTF-8 itself. Exact versions: portable-pty 0.9, @xterm/xterm 6.0.0, @xterm/addon-fit 0.11.0, @xterm/addon-web-links 0.12.0 (stable, not the 6.1 betas Termul pins).

## Konsekuensi

About 150 lines of pty code owned here instead of borrowed; ConPTY behavior must be tested on Windows (CI workflow). No session persistence across app restarts (non-goal).

## Agent + model

claude-code / claude-opus-5-5
