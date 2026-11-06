---
task_id: 20261106-mcp-protocol
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

First of three M3-lite PRs: the `tempa mcp` stdio server with the protocol core and the read tools (tempa_context, tempa_status, tempa_check, tempa_pr_body). Spec: docs/superpowers/specs/2026-11-06-tempa-m3-lite-design.md. Decision: docs/decisions/2026-11-06-mcp-server-on-serde-json.md.

## Langkah

1. JSON-RPC core: initialize (version negotiation), ping, tools/list, tools/call; -32700, -32600, -32601; no reply to notifications.
2. Read tools; tempa_check runs harness commands only with harness: true.
3. CLI `tempa mcp`: stdio loop from the git toplevel, replies only on stdout.

## File yang disentuh

crates/tempa-core/src/{mcp.rs,lib.rs,task.rs}, crates/tempa-core/tests/mcp.rs, crates/tempa-cli/{src/main.rs,tests/cli.rs,Cargo.toml}, docs/decisions/.

## Risiko

Anything other than JSON on stdout breaks MCP clients (harness output is captured). Long harness runs would hit client tool timeouts, so they are opt-in. Not registered in agent configs yet (PR3).
