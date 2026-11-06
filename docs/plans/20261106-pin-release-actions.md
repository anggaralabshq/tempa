---
task_id: 20261106-pin-release-actions
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Harden the release workflow (security review finding): it holds publish tokens, so third-party actions must not float on tags and secrets must not reach steps that do not need them.

## Langkah

1. Pin actions/checkout, taiki-e/create-gh-release-action, taiki-e/upload-rust-binary-action and dtolnay/rust-toolchain to commit SHAs (tag in a comment).
2. Workflow default permissions contents: read; write only for the release and upload jobs.
3. CARGO_REGISTRY_TOKEN and HOMEBREW_TAP_TOKEN only in their publish steps; the steps skip themselves when the secret is empty.

## File yang disentuh

.github/workflows/release.yml (protected; owner approved the release workflow).

## Risiko

Pinned actions do not get fixes automatically; bump the SHAs when updating (Dependabot can do it).
