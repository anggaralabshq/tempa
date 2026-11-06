## Context

The desktop app (app/) was frozen when tempa refocused on contract sync. Kept in main it adds 7.7k lines, Bun and Tauri to the pre-push harness, and a Windows-only CI job, and confuses readers of a CLI repo.

## Options

1. Keep app/ in main, frozen. 2. Move it to a branch and make main CLI-only. 3. Delete it.

## Decision

Option 2. Branch desktop-app holds the app as of m4. main drops app/, its harness commands and app-windows.yml; ci.yml tests the CLI on Linux, macOS and Windows for pull requests into main. The removal exceeds max_diff_lines and is pushed past the pre-push gate once, because it only deletes code that is archived on the branch.

## Consequences

Pre-push needs only Rust. The app can return from its branch later. The README points at the branch.

## Agent + model

claude-code / claude-opus-5-5
