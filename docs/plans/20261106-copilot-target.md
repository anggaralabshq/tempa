---
task_id: 20261106-copilot-target
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

m4 T2: GitHub Copilot as the sixth target, from GitHub's and VS Code's current docs (verified 2026-11-06). VS Code now runs the same Copilot harness as the CLI, so the target follows the CLI/harness formats.

## Langkah

1. Agent::Copilot ("copilot"), enabled by default for new repos.
2. Context: none; Copilot reads AGENTS.md natively.
3. Commands -> user-invoked skills .github/skills/<name>/SKILL.md (disable-model-invocation: true); a name taken by a real skill is a gap.
4. Agents -> .github/agents/<stem>.agent.md; skills via .agents/skills; MCP -> .github/mcp.json (type local/http, tools ["*"]).
5. settings-deny is a gap (no repo-level deny). Matrix column, README and AGENTS.md agent lists.

## File yang disentuh

crates/tempa-core/src/{config,targets,sync}.rs, crates/tempa-core/tests/{targets,config}.rs, docs/target-matrix.md, README.md, AGENTS.md.

## Risiko

Existing repos list their agents explicitly in tempa.toml, so Copilot stays off until added. Visual Studio, JetBrains and Xcode chat read only copilot-instructions.md and get no context; doctor keeps reporting that file as legacy. With Claude on, the Copilot CLI also sees .claude/skills and .mcp.json (identical content).
