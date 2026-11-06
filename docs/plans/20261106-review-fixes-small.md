---
task_id: 20261106-review-fixes-small
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Fix the small findings from the m4 review: TOML escaping, Codex agent description, Cursor agents with Codex, detached-HEAD task base, pr-body base, one Indonesian message.

## Steps

- `toml_basic` escapes through toml_edit, so multi-line descriptions give valid TOML (Codex agents, Gemini commands).
- Codex agent `description` falls back to the name (Codex requires it).
- Cursor gets `.cursor/agents/` unless Claude is enabled (it reads `.codex/agents/` only as markdown); target matrix updated.
- `task new` does not record a detached HEAD as the base; pr-body's re-verify line uses the task base.
- Translate the Bitbucket shallow-clone hint.

## Files touched

crates/tempa-core/src/{targets,task,check}.rs, crates/tempa-core/tests/{targets,task}.rs, docs/target-matrix.md

## Risks

Repos with Codex and Cursor but not Claude get new `.cursor/agents/` files on the next sync; Cursor gives `.cursor/` precedence on equal names, so nothing is duplicated.
