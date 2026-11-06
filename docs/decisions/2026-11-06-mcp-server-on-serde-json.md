## Context

Agents other than Claude Code cannot load tempa's commands, subagents or deny rules natively, and each agent writes plans and changes entries its own way. M3-lite needs a way for every agent to read the contract and write workflow documents that are validated as they are written, without the M2 desktop host.

## Options

1. A stdio MCP server inside the tempa binary, hand-rolled on serde_json (already a dependency).
2. The same server on an MCP SDK crate (rmcp): more protocol coverage, but a new dependency tree and an async runtime.
3. No server: keep only generated files plus `tempa check` in hooks.

## Decision

Option 1. `tempa mcp` speaks newline-delimited JSON-RPC 2.0 with the tools subset of MCP; `tempa sync` injects it into every agent's MCP config at render time (`[mcp] serve`, default true), never into `.tempa/mcp.toml`. Public surface added: the `tempa mcp` command, the `[mcp]` table, the seven `tempa_*` tools and `drift.foreign`. serde_json is added as a dev-dependency of tempa-cli for the stdio test only.

## Consequences

No new runtime dependency and one binary to ship. tempa owns protocol conformance: only tools are implemented, and newer MCP features (resources, progress, cancellation) need code here. Long harness runs are opt-in on `tempa_check` because clients time out tool calls. Repos with hand-made MCP config files must run `tempa import --yes` once on upgrade (`drift.foreign`).

## Agent + model

claude-code / claude-opus-5-5
