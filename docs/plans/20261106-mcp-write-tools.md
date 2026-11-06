---
task_id: 20261106-mcp-write-tools
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Second of three M3-lite PRs: contract-validated write tools tempa_plan, tempa_decide and tempa_log_change, so plans, decisions and changes entries come out the same whatever agent writes them.

## Langkah

1. md::set_section: replace one section body (fence-aware, case-insensitive).
2. tempa_plan: only [headings].plan keys, non-empty string values, no #/## headings or unclosed fences in bodies; frontmatter never written, so agents cannot approve.
3. tempa_decide: every decision heading required before the file is created.
4. tempa_log_change: wraps tempa log and its validation.

## File yang disentuh

crates/tempa-core/src/{mcp.rs,md.rs}, crates/tempa-core/tests/mcp.rs.

## Risiko

A body that ends a section early would corrupt the plan (rejected before writing). Agents may try to change state or approval (unknown section, rejected). Agents may still edit an approved plan (deferred; plan_before_edit is not enforced yet).
