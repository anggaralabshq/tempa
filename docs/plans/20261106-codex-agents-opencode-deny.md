---
task_id: 20261106-codex-agents-opencode-deny
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

m4 T1b: close two gaps that now have an official project-level option (verified 2026-11-06): Codex subagents and OpenCode edit permissions.

## Langkah

1. Codex: .codex/agents/<stem>.toml with name, description, developer_instructions (the agent body).
2. Cursor skips .cursor/agents when Codex is on too (Cursor reads .codex/agents).
3. OpenCode: opencode.json permission.edit keeps the user's default under "*" and denies each protected glob; doctor shows "permission deny".
4. target-matrix.md: rows and sources; Codex and Gemini deny stay gaps with the reason.

## File yang disentuh

crates/tempa-core/src/{targets,doctor}.rs, crates/tempa-core/tests/{targets,doctor,review_fixes}.rs, docs/target-matrix.md, repo outputs (opencode.json, .tempa/lock.json).

## Risiko

OpenCode globs relative to the project root are not stated in its docs; if they are not, the deny rules match nothing and CI stays the guard. serde_json sorts keys, so a protected glob starting with a symbol before "*" would sort ahead of the default.
