---
task_id: 20261106-m3-acp-client
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

M3 task 2a/7: an ACP client: start an agent subprocess, JSON-RPC both ways, handshake with the tempa MCP server injected, prompt, cancel, permission requests to a handler, model choices, and ending the whole process tree.

## Langkah

1. acp/conn.rs: spawn (own process group on Unix), request with timeout, notify, agent requests on their own thread, dead agent fails pending calls with its stderr tail, stop kills the tree.
2. acp/session.rs: initialize + session/new (mcpServers = tempa), prompt, cancel, stop, models from configOptions (category model) or the unstable models field, set_model.
3. tests/fake-agent.mjs and tests against it.

## File yang disentuh

app/src-tauri/src/acp/{mod.rs,conn.rs,session.rs,tests.rs}, app/src-tauri/tests/fake-agent.mjs, app/src-tauri/src/lib.rs, docs/decisions/.

## Risiko

Not wired into the UI yet (next PR adds agents and chat commands). Real adapters may send fields the schema marks unstable; unknown messages are ignored.
