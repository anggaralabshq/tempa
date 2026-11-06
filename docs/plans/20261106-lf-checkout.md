---
task_id: 20261106-lf-checkout
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Source files keep LF line endings in every checkout, so templates compiled with include_str! and test fixtures are the same on Windows.

## Steps

- `.gitattributes`: `* text=auto eol=lf`.
- Reproduce with a `core.autocrlf=true` clone (fails before, passes after).

## Files touched

.gitattributes

## Risks

None for users of released binaries; contributors on Windows get LF files.
