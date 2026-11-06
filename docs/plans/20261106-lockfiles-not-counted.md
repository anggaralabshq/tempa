---
task_id: 20261106-lockfiles-not-counted
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Dependency lockfiles (Cargo.lock, bun.lock, package-lock.json, ...) no longer count toward max_diff_lines: they are generated and not reviewed, and the M2 app scaffold would otherwise fail the gate on its lockfiles alone. Also adds the approved M2 design and plan.

## Langkah

1. Test diff_gate_ignores_lockfiles (RED: 620 lines counted).
2. check::LOCKFILES; gitx::diff_lines skips by file name in any directory.
3. README note.

## File yang disentuh

crates/tempa-core/src/{check.rs,gitx.rs}, crates/tempa-core/tests/check.rs, README.md, docs/superpowers/specs/2026-11-06-tempa-m2-desktop-design.md, docs/superpowers/plans/2026-11-06-tempa-m2-desktop.md.

## Risiko

A hand-edited lockfile change is no longer counted; the change still shows in the PR diff and the new-dependency decision gate still fires on manifests.
