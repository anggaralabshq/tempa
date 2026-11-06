---
task_id: 20261106-check-task-base
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

m4 T4: tempa check measures a task against the branch it was started from, so tasks on a milestone branch (m4) pass the hook without --no-verify. Also: doctor's lifecycle column says "off" instead of the frozen "M3", and this repo's hooks match the English templates.

## Langkah

1. tempa task new records the current branch in repo-local git config (branch.<task>.tempaBase).
2. gitx::base_name(root, cfg, explicit): explicit, CI PR destination, recorded task base, then [ci].base.
3. doctor lifecycle "M3" -> "off".
4. Copy templates/pre-commit and pre-push into .tempa/hooks.

## File yang disentuh

crates/tempa-core/src/{gitx,check,task,doctor}.rs, crates/tempa-core/tests/{task,registration}.rs, .tempa/hooks/{pre-commit,pre-push}.

## Risiko

Repo-local config is per clone: a task branch checked out on another machine falls back to [ci].base (the old behaviour). CI is unaffected (it uses the PR destination).
