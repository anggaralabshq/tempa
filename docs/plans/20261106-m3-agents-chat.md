---
task_id: 20261106-m3-agents-chat
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

M3 task 2b/7: the five v1 agents (launch commands pinned from the ACP registry) and live chats for the UI: Tauri commands to list agents, start a chat in a worktree or project root with the tempa MCP server injected, prompt, cancel, stop, pick a model and answer permission requests.

## Langkah

1. acp/agents.rs: AGENTS table, PATH lookup with PATHEXT, resolve with an install hint.
2. acp/chat.rs: ChatManager; events update/permission/turn_end/error/exit; cancel and stop answer pending permissions `cancelled`; stop ends the process tree even mid-turn.
3. lib.rs: agent_list, chat_start (MCP = this binary in --mcp mode), chat_prompt/cancel/stop/set_model/permission; app exit stops all chats.

## File yang disentuh

app/src-tauri/src/acp/{agents.rs,chat.rs,mod.rs,tests.rs}, app/src-tauri/src/lib.rs.

## Risiko

Every permission request still goes to the user until the tool policy lands (task 4). Adapter versions are pinned; updating them is a code change.
