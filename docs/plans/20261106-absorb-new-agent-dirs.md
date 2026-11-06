---
task_id: 20261106-absorb-new-agent-dirs
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Hand-written files in the directories tempa now owns (`.github/agents`, `.codex/agents`, `.github/skills`) can be moved into the source instead of blocking `tempa check`.

## Steps

- `tempa import` absorbs `.github/agents/<x>.agent.md` as `.tempa/agents/<x>.md`, and `.codex/agents/<x>.toml` as a markdown subagent (name, description, developer_instructions).
- `.github/skills` stays out of import (tempa writes skills to `.agents/skills`); its untracked finding says to move the skill to `.tempa/skills/<name>/`.
- Regression tests in review_fixes.rs.

## Files touched

crates/tempa-core/src/{import,sync,targets}.rs, crates/tempa-core/tests/review_fixes.rs

## Risks

A Codex agent TOML without developer_instructions is reported as a gap and left in place.
