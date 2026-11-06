---
task_id: 20261106-readme-demo
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Show what tempa does in the first screen of the README: a terminal demo, recorded reproducibly.

## Steps

- `docs/demo/setup.sh` builds a small Claude Code repo; `docs/demo/demo.tape` (vhs) records import, doctor, and a direct edit being detected and absorbed.
- Fix the doctor table: the protected column is as wide as its longest value, so `lifecycle` lines up (regression check in the CLI test).
- Embed `docs/demo/demo.gif` under the README tagline.

## Files touched

docs/demo/{setup.sh,demo.tape,demo.gif}, README.md, crates/tempa-cli/src/main.rs, crates/tempa-cli/tests/cli.rs

## Risks

The GIF goes stale when output changes; re-record with `vhs docs/demo/demo.tape`.
