---
task_id: 20261106-m3-host-mcp-verify
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

M3 task 1/7: the app binary serves the injected tempa MCP server (`tempa-app --mcp <root> --verify <file>`); a harness check from an agent or the Verify button is run and recorded by the host in verify.json and moves the task state (pass → record, fail → implement). Includes the M3 spec and plan.

## Langkah

1. tempa-core: task::STATES and task::set_state.
2. app verify.rs: run (check with harness, write verify.json, move state), load, file path per project/task.
3. app mcp_host.rs: handle_line routes tempa_check {harness:true} to verify, the rest to tempa_core::mcp; serve on stdio; main.rs mode switch.
4. verify_result command for the Task panel.

## File yang disentuh

crates/tempa-core/src/task.rs, crates/tempa-core/tests/task.rs, app/src-tauri/src/{verify.rs,mcp_host.rs,projects.rs,contract.rs,lib.rs,main.rs}, docs/superpowers/{specs,plans}/, docs/decisions/.

## Risiko

verify.json lives outside the repo and can be edited by anything with disk access; it guides the app, CI stays the gate. FindingDto.severity becomes a String (same JSON).
