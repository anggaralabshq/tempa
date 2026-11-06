---
task_id: 20261106-mcp-import-fixes
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

`tempa import` keeps every MCP server a repo already has: servers in `.github/mcp.json` are absorbed, and Cursor's variables survive a round trip.

## Steps

- Regression tests: a Copilot http server in `.github/mcp.json` reaches `.tempa/mcp.toml` and every agent; a Cursor server with `${workspaceFolder}` and `${env:DB_URL}` comes back unchanged for Cursor and as `${DB_URL}` / `{env:DB_URL}` elsewhere.
- import: read `.github/mcp.json` (`mcpServers`); accept `tools: ["*"]`; turn Cursor `${env:X}` into `${X}`.
- targets: `cursor_env` leaves Cursor's built-in variables alone (cursor.com/docs/context/mcp).
- Target matrix notes.

## Files touched

crates/tempa-core/src/{import,targets}.rs, crates/tempa-core/tests/review_fixes.rs, docs/target-matrix.md

## Risks

A source that used `${workspaceFolder}` as a real environment variable name now reaches Cursor unprefixed; that name is reserved by Cursor anyway.
