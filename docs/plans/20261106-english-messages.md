---
task_id: 20261106-english-messages
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

All user-facing text in English: CLI messages and help, hook scripts, generated tempa.toml comments, the CI handoff doc, README, and the GitHub repo About.

## Langkah

- Translate every message string in tempa-core and tempa-cli; update tests that asserted Indonesian text.
- `--harness` takes pass|fail|pending; Indonesian values from 0.1 stay accepted and are normalized.
- CI handoff doc rewritten for any CI (Bitbucket Pipelines and GitHub Actions examples).
- README rewritten in English; repo About set on GitHub.

## File yang disentuh

crates/tempa-core/src/*.rs, crates/tempa-core/src/templates/{pre-commit,pre-push,devops.md}, crates/tempa-cli/src/main.rs, tests that match message text, README.md, AGENTS.md, docs/tempa-devops.md.

## Risiko

Scripts that grep tempa's Indonesian messages break; finding codes are unchanged, so scripts should match on codes. Contract vocabulary (required headings, changes fields, pr-body sections) is unchanged on purpose.
