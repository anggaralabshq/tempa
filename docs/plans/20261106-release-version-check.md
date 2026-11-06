---
task_id: 20261106-release-version-check
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

A release tag that does not match the crate version stops the release before anything is published.

## Steps

- `scripts/check-version.sh TAG`: the tag equals `v` + the workspace version, and tempa-cli's `tempa-core` requirement equals it too.
- release.yml runs it first in `create-release`; docs/releasing.md says so.
- Checked by hand: v0.1.0 passes; v0.1.1, 0.1.0 and a mismatched tempa-core requirement fail.

## Files touched

scripts/check-version.sh, .github/workflows/release.yml, docs/releasing.md

## Risks

None for code; release.yml is a protected path, changed with the owner's approval.
