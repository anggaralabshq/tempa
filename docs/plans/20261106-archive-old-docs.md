---
task_id: 20261106-archive-old-docs
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Keep the docs a reader of the public repo needs; leave the development record in git history.

## Steps

- Delete `docs/superpowers/` (internal design and plan drafts) and the 30 Indonesian task plans; the English plans of this milestone stay as examples.
- Delete the six decision records about the desktop app (they live on branch `desktop-app`); translate the headings of the rest.
- CHANGES.md keeps the English entries only.

## Files touched

docs/superpowers/, docs/plans/, docs/decisions/, CHANGES.md

## Risks

Docs only. Everything removed stays in git history and on the `desktop-app` / `m3` branches.
