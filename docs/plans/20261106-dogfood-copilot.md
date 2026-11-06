---
task_id: 20261106-dogfood-copilot
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Dogfood the new GitHub Copilot target in tempa's own repo (owner approved editing the protected tempa.toml).

## Langkah

1. tempa.toml [agents].enabled gains "copilot".
2. tempa sync writes .github/skills/review/SKILL.md and .github/mcp.json.

## File yang disentuh

tempa.toml (protected; owner approved), .github/skills/review/SKILL.md, .github/mcp.json, .tempa/lock.json.

## Risiko

None beyond the generated files; .github/workflows is untouched (managed dirs are .github/agents and .github/skills only).
