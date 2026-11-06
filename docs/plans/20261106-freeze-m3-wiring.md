---
task_id: 20261106-freeze-m3-wiring
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Refocus tempa on its original idea, one contract kept in sync with each agent's native format (owner decision 2026-11-06). M3 (agent chat, tool policy, worktrees) is frozen on branch m3 and removed from main in three PRs; this is the first.

## Langkah

1. Bring the once-per-milestone CI trigger (PR #20, merged to m3 only) to this line.
2. Restore lib.rs, main.rs, contract.rs, projects.rs to their M2 state (PR #15).
3. Delete acp/{mod,chat,agents}.rs, mcp_host.rs, verify.rs. conn.rs/session.rs/tests.rs stay unreferenced until R2.
4. App tests and clippy green.

## File yang disentuh

.github/workflows/app-windows.yml (protected; owner approved the CI change), app/src-tauri/src/{lib,main,contract,projects}.rs, app/src-tauri/src/{acp/mod,acp/chat,acp/agents,mcp_host,verify}.rs, docs/decisions/.

## Risiko

None for users: M3 had no UI on main. Spec, plan, CHANGES entries and decision records of M3 stay as history; the code lives on m3.
