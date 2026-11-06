# Tempa M3-lite (`tempa mcp`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** A stdio MCP server, `tempa mcp`, that every agent launches itself, serving the contract (`tempa_context`) and contract-validated writes (plan, decision, changes), registered automatically by `tempa sync`.

**Architecture:** `tempa-core/src/mcp.rs` holds the JSON-RPC handling (`handle`) and the tools; it calls existing `task`, `check`, `source`, `md` functions. `tempa-cli` adds `tempa mcp`, a loop over stdin lines. `targets::render` injects a `tempa` server into each agent's MCP config unless `[mcp] serve = false`.

**Tech Stack:** Rust 2024, serde_json (no new dependency).

**Spec:** `docs/superpowers/specs/2026-11-06-tempa-m3-lite-design.md` (approved), main spec section 7.

## Global Constraints

- MSRV 1.85; `cargo clippy -D warnings`, `cargo fmt --check` clean.
- No new dependency.
- All user-facing text in English (AGENTS.md convention). Contract vocabulary unchanged.
- Tools never write outside the project root and never touch generated files.
- Agents never approve plans: no tool changes `state`, `approved_by`, `approved_at`.
- Validation failures are tool results with `isError: true` and a fix-it message, not JSON-RPC errors.

## Review Focus

1. **Malformed input on stdin** (invalid JSON, a batch array, missing `id`, unknown method, a notification with an unknown method): the server answers with the right JSON-RPC error (or nothing for notifications) and keeps running.
2. **Tool called off a task branch** (`tempa_plan`, `tempa_log_change` on `main`): clear `isError` result naming `tempa task new`, no file written.
3. **Partial plan writes**: `tempa_plan` with only some sections updates those and leaves the rest; unknown section names are rejected before anything is written; the frontmatter (state/approval) is preserved byte-for-byte.
4. **User already defines a server named `tempa`** in `.tempa/mcp.toml`: theirs wins, nothing injected; `tempa import` never copies the injected server back into `.tempa/mcp.toml`.
5. **`[mcp] serve = false`**: no `tempa` entry in any agent config, and turning it off removes the entry on the next sync without touching other keys.

---

### Task 1: Protocol core

**Files:** Create `crates/tempa-core/src/mcp.rs`; modify `crates/tempa-core/src/lib.rs`; test `crates/tempa-core/tests/mcp.rs`.

**Produces:**
- `pub const PROTOCOL_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"]`
- `pub fn handle(root: &Path, msg: &serde_json::Value) -> Option<serde_json::Value>` — `None` for notifications.
- `pub fn handle_line(root: &Path, line: &str) -> Option<String>` — parse error → `-32700`, batch array → `-32600`.

Tests: `initialize_negotiates_version` (requested supported version echoed; unknown → `PROTOCOL_VERSIONS[0]`; result has `serverInfo.name == "tempa"` and `capabilities.tools`), `notifications_get_no_reply`, `ping_returns_empty_result`, `unknown_method_is_32601`, `garbage_is_32700_and_batch_is_32600`, `tools_list_names` (exactly the seven tool names, each with an `inputSchema` of type object).

### Task 2: Read tools

**Produces:** tool handlers `tempa_context`, `tempa_status`, `tempa_check`, `tempa_pr_body` inside `mcp.rs`.

- `tempa_context {kind?, name?}`: `kind` ∈ `all|commands|agents|skills|rules` (default `all`). Text: AGENTS.md; commands and subagents (name + description; full body when `name` given); skills (name + description; SKILL.md body when `name` given); rules (protected paths, harness commands, required headings per document); active plan when on a task branch. Unknown `name` → `isError`.
- `tempa_status {}`: JSON text `{ "task_id", "branch", "plan", "state", "changes_entries", "drift": [..] }`; off a task branch `task_id` is null.
- `tempa_check {contract_only?}`: JSON text array of findings `{code, severity, path, message}`; `isError` true when any error-severity finding.
- `tempa_pr_body {}`: the body text; off a task branch → `isError`.

Tests (in `tests/mcp.rs`): `context_serves_commands_codex_cannot_read` (repo with `.tempa/commands/review.md`: `tempa_context {kind:"commands", name:"review"}` returns its body), `context_rules_lists_protected_paths`, `status_off_task_branch`, `check_returns_structured_findings`.

### Task 3: Write tools

**Produces:** `tempa_plan {sections}`, `tempa_decide {slug, sections}`, `tempa_log_change {summary, reason, decision?, harness, agent}`; `pub fn set_section(md: &str, title: &str, body: &str) -> Option<String>` in `md.rs` (replace the body under an existing heading, case-insensitive; `None` if the heading is absent).

- `tempa_plan`: off a task branch → `isError`. Every key must be one of `[headings].plan` (case-insensitive) else `isError` listing allowed names, nothing written. After applying, required headings still empty are listed in the result text (not an error: plans fill in over time).
- `tempa_decide`: all `[headings].decisions` must be non-empty after applying `sections`, else `isError` and no file created. Uses `task::decide` for the path and template.
- `tempa_log_change`: wraps `task::log`; its errors become `isError`.

Tests: `plan_updates_given_sections_only` (frontmatter identical before/after), `plan_rejects_unknown_section_without_writing`, `plan_off_task_branch_is_error`, `decide_requires_all_sections`, `log_change_validates_one_line`, `set_section_unit` (md.rs unit test).

### Task 4: Registration, doctor, init, import

**Files:** `config.rs` (`[mcp] serve`, default true), `targets.rs`, `doctor.rs`, `init.rs`, `import.rs`.

- Render: effective MCP servers = `.tempa/mcp.toml` servers + `tempa` (`command = "tempa"`, `args = ["mcp"]`) when `serve` and no user server named `tempa`. Claude `.claude/settings.json` additionally gets `"enabledMcpjsonServers": ["tempa"]` merged into any existing list, so Claude Code does not prompt for the project server.
- `doctor`: `lifecycle` = `"mcp"` when the agent's config carries the tempa server, else `"M3"`.
- `init`: AGENTS.md template starts with `Agents: use the tempa_* MCP tools (tempa_context, tempa_plan, tempa_log_change, tempa_check) for plans, decisions, changes and checks.`
- `import`: skip a server named `tempa` whose command is `tempa`.

Tests: `sync_registers_tempa_server_everywhere` (five configs contain it: `.mcp.json`, `.cursor/mcp.json`, `.gemini/settings.json`, `opencode.json`, `.codex/config.toml`), `serve_false_removes_entry_keeps_other_keys`, `user_server_named_tempa_wins`, `import_does_not_copy_injected_server`, `doctor_lifecycle_mcp`. Existing tests that assume no MCP files are updated to `serve = false` or the new expectation.

### Task 5: CLI and pilot

**Files:** `crates/tempa-cli/src/main.rs`, `crates/tempa-cli/tests/cli.rs`, `docs/target-matrix.md`, `README.md`.

- `tempa mcp`: root = `git rev-parse --show-toplevel` when inside a repo, else cwd; read stdin line by line, write each reply plus `\n`, flush; exit 0 on EOF; never print anything else to stdout (diagnostics to stderr).
- CLI test `mcp_stdio_roundtrip`: spawn `tempa mcp`, send initialize + initialized + tools/call `tempa_status`, read two replies.
- Pilot: `claude -p` in this repo calls `tempa_context` (prove with the answer); `opencode mcp list` shows `tempa`.
- Docs: README section "MCP tools"; target-matrix note on the injected server.

Verification for every task: `cargo test --workspace`; at the end also clippy, fmt, e2e.
