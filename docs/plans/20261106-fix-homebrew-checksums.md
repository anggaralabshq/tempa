---
task_id: 20261106-fix-homebrew-checksums
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

The Homebrew formula reads the checksum files the release actually uploads, and fails instead of publishing empty checksums.

## Steps

- Read `tempa-TAG-TARGET.sha256` (the name taiki-e/upload-rust-binary-action uses), not `.tar.gz.sha256`.
- Compute all four sums before printing; exit 1 on a missing file or a value that is not 64 hex characters.
- Check by hand: missing file, empty file, and valid files (output passes `ruby -c`).

## Files touched

scripts/homebrew-formula.sh

## Risks

None beyond the release job: a bad checksum now stops the job before anything is pushed to the tap.
