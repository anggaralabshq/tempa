---
task_id: 20261106-name-collisions
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

No source item is dropped silently when two of them map to the same Codex agent, Copilot agent or Copilot skill.

## Steps

- Codex and Copilot agent files are named from the path (`backend/reviewer.md` → `backend-reviewer`).
- Copilot command skills: names compared lowercase against source skills and earlier commands; a collision or an empty name is a `commands` gap.
- Tests in targets.rs.

## Files touched

crates/tempa-core/src/targets.rs, crates/tempa-core/tests/targets.rs

## Risks

Agents in subfolders get new Codex/Copilot file names on the next sync; the old files are removed by sync as stale outputs.
