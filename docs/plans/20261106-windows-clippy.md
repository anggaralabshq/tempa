---
task_id: 20261106-windows-clippy
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Clippy passes on Windows.

## Steps

- `harness_env.rs` is `#![cfg(unix)]` as a whole, so its imports are not unused on Windows.
- Check locally with `cargo clippy --target x86_64-pc-windows-gnu`.

## Files touched

crates/tempa-core/tests/harness_env.rs

## Risks

None; test-only.
