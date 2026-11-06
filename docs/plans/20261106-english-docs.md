---
task_id: 20261106-english-docs
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Move this repo's own contract and docs to English, now that English is the default vocabulary.

## Steps

- Translate AGENTS.md (Stack/Commands/Conventions/Off-limits), HARNESS.md (Definition of done/Gates) and docs/target-matrix.md.
- HARNESS.md: describe CI as once per milestone; mark `app/` frozen in AGENTS.md.
- Translate `.tempa/commands/review.md` and the Dockerfile comment, then `tempa sync`.

## Files touched

AGENTS.md, HARNESS.md, docs/target-matrix.md, .tempa/commands/review.md, generated review targets, Dockerfile

## Risks

Docs only. Old CHANGES.md entries and tempa.toml comments stay as written (history; tempa.toml is protected).
