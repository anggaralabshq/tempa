---
task_id: 20261106-cursor-native
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

m4 T1a: align the Cursor target with Cursor's current docs (verified 2026-11-06): it reads AGENTS.md natively, expands MCP variables as ${env:VAR}, has project subagents, and its CLI honours a project deny list.

## Langkah

1. Stop generating .cursor/rules/tempa.mdc (sync removes an untouched old copy).
2. .cursor/mcp.json: ${VAR} -> ${env:VAR} in every server string.
3. .cursor/agents/<rel> with name/description, only when Claude is off (Cursor also reads .claude/agents).
4. .cursor/cli.json permissions.deny Write(<glob>) per protected path, merged with the user's keys; doctor shows "cli deny (IDE: CI)".
5. Tests and e2e moved off tempa.mdc onto a command source; target-matrix.md updated with sources.

## File yang disentuh

crates/tempa-core/src/{targets,sync,doctor}.rs, crates/tempa-core/tests/{targets,sync,review_fixes,doctor}.rs, scripts/e2e.sh, docs/target-matrix.md, .cursor/cli.json, .cursor/rules/tempa.mdc (removed), .tempa/lock.json.

## Risiko

Cursor IDE may ignore .cursor/cli.json (only the CLI is documented); CI stays the guard. Users who edited tempa.mdc by hand keep their file (sync never deletes edited outputs).
