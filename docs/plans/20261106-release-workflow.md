---
task_id: 20261106-release-workflow
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Release and distribution (owner approved the protected workflow file and the tap repo): a version tag builds binaries for macOS, Linux and Windows, attaches them with checksums to a GitHub Release, publishes tempa-core and tempa-cli to crates.io, and writes the Homebrew formula.

## Langkah

1. .github/workflows/release.yml on tags v*: taiki-e create/upload actions (5 targets, sha256), crates-io and homebrew jobs that skip without their secrets.
2. scripts/homebrew-formula.sh renders Formula/tempa.rb from the .sha256 files (checked with ruby -c on fake sums).
3. Repo anggaralabshq/homebrew-tap created, private until launch.
4. docs/releasing.md (secrets, steps); README Install section.

## File yang disentuh

.github/workflows/release.yml (protected; owner approved), scripts/homebrew-formula.sh, docs/releasing.md, README.md.

## Risiko

The workflow cannot run before the first tag; the first release is its test. brew install works for others only after this repo and the tap are public. Compute runs only on tags.
