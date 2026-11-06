<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/banner-dark.png">
  <img src="docs/assets/banner-light.png" width="100%" alt="tempa: one context contract for every coding agent">
</picture>

### One context contract and one workflow for every coding agent

**[Quick start ↓](#quick-start)** &nbsp;·&nbsp; **[Install ↓](#install)** &nbsp;·&nbsp; **[Agent support →](docs/target-matrix.md)** &nbsp;·&nbsp; **[Releases →](https://github.com/anggaralabshq/tempa/releases)**

[![CI](https://github.com/anggaralabshq/tempa/actions/workflows/ci.yml/badge.svg?event=pull_request)](https://github.com/anggaralabshq/tempa/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/tempa-cli?label=crates.io)](https://crates.io/crates/tempa-cli)
[![Release](https://img.shields.io/github/v/release/anggaralabshq/tempa?logo=github)](https://github.com/anggaralabshq/tempa/releases)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

</div>

![tempa importing a Claude Code repo, reporting per-agent parity, and absorbing a direct edit back into the source](docs/demo/demo.gif)

Teams rarely use a single coding agent. One engineer lives in Claude Code, another in Codex, Gemini CLI, OpenCode, Cursor or GitHub Copilot, and each reads a different set of files. Project knowledge piles up in one agent's format (usually `CLAUDE.md` and `.claude/`), and teammates on other agents work without it: they miss the conventions, run the wrong commands, and output quality depends on which agent wrote the code.

tempa keeps the contract in one neutral place and makes every agent read it, then holds every change to the same workflow:

- **One source.** `AGENTS.md`, `.tempa/` (commands, subagents, skills, MCP servers) and `tempa.toml`. tempa generates what each agent reads: `CLAUDE.md`, `.claude/`, `.gemini/`, `.opencode/`, `opencode.json`, `.cursor/`, `.codex/config.toml`, `.agents/skills/`.
- **No drift.** A lock file records what tempa wrote. Direct edits to generated files are detected, and can be absorbed back into the source instead of being lost.
- **One workflow.** Every change is a task with a plan, an optional decision record, a changes entry naming the agent and model used, and a pull request body with the same structure, whichever agent did the work.
- **Gates before CI.** Git hooks sync and check locally; the same `tempa check` runs in CI when you add it.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/architecture-dark.png">
  <img src="docs/assets/architecture-light.png" width="100%" alt="AGENTS.md and .tempa/ go into tempa, which syncs every agent's files (Claude Code, Codex, Gemini CLI, OpenCode, Cursor, GitHub Copilot) and absorbs direct edits back into the source">
</picture>

Supported agents: Claude Code, Codex, Gemini CLI, OpenCode, Cursor, GitHub Copilot. What each one supports, and where tempa has to fall back, is in [docs/target-matrix.md](docs/target-matrix.md); `tempa doctor` reports it per repo.

## Quick start

**1. Install**

```sh
brew install anggaralabshq/tap/tempa     # or cargo, or a binary: see Install below
```

**2. Adopt it in your repo**

```sh
tempa import          # dry run: shows what would be absorbed from CLAUDE.md, .claude/, .cursorrules, ...
tempa import --yes    # writes AGENTS.md, .tempa/, tempa.toml, hooks; syncs every agent
```

No agent files yet? Run `tempa init` instead (`tempa.toml`, `AGENTS.md`, `HARNESS.md`, `CHANGES.md`, git hooks).

**3. See what every agent gets**

```sh
tempa doctor          # generated files, gaps, protected paths, hook status per agent
```

Each engineer runs `tempa init` once per clone to install the hooks (it never overwrites existing files).

## Install

```sh
brew install anggaralabshq/tap/tempa     # macOS, Linux
cargo install tempa-cli --locked         # any platform with Rust 1.85+
```

Or download a binary for macOS, Linux or Windows from [Releases](https://github.com/anggaralabshq/tempa/releases). tempa needs git on PATH. From a clone: `cargo install --path crates/tempa-cli --locked`.

## Daily workflow

```sh
tempa task new add-invoice     # branch tempa/<date>-add-invoice + plan file
# fill in the plan, then a human approves it:
tempa task approve
tempa decide use-sqlite        # optional decision record
# ...work with any agent...
tempa log --summary "Add invoice endpoint" --reason "Billing v2" \
          --harness pass --agent "codex / gpt-5"
tempa pr-body                  # paste into the pull request
```

## Commands

| Command | What it does |
|---|---|
| `tempa init` | Create missing contract files and git hooks. Refuses repos that already have agent context; use `import` there. |
| `tempa import [--yes]` | Absorb existing agent files into the neutral source, keep other keys of shared configs, then sync. |
| `tempa sync` | Generate every agent's files. Never overwrites a file tempa did not write or one edited directly. |
| `tempa sync --check` | Report drift without writing (for CI). |
| `tempa sync --fix` | Pre-commit hook: sync from staged sources, stage outputs and the lock, block on drift. |
| `tempa sync --force` | Overwrite generated files on purpose. |
| `tempa check [--contract-only] [--base <branch>]` | Drift, contract documents, task rules, then the `[harness]` commands. |
| `tempa doctor` | Per-agent parity: generated files, gaps, protected paths, legacy files, hook status. |
| `tempa task new <slug>` / `tempa task approve` | Start a task; mark its plan approved. |
| `tempa decide <slug>` | Create a decision record. |
| `tempa log ...` | Append a changes entry for the current task. |
| `tempa pr-body` | Print the pull request body for the current task. |
| `tempa mcp` | Run the MCP server on stdio (agents start it from their MCP config). |

Exit codes: `0` ok, `1` findings, `2` error.

## MCP tools

`tempa sync` registers a `tempa` MCP server (`tempa mcp`, stdio) in every agent's MCP config, so each agent can read the contract and write workflow documents that are validated as they are written. Turn it off with `[mcp] serve = false` in `tempa.toml`. `tempa` must be on the `PATH` of the agent process.

Upgrading a repo that already has its own `.mcp.json`, `.cursor/mcp.json`, `opencode.json` or `.codex/config.toml` (files tempa did not write): `tempa sync` reports them as `drift.foreign` and the pre-commit hook blocks until you run `tempa import --yes`, which moves their servers into `.tempa/mcp.toml` and keeps their other keys.

| Tool | What it does |
|---|---|
| `tempa_context` | AGENTS.md, commands, subagents, skills, rules (protected paths, harness, required headings) and the active plan. Agents that cannot load a command or subagent natively read it here. |
| `tempa_status` | Current task, plan state, changes entries, drift (JSON). |
| `tempa_plan` | Fill sections of the current plan. Unknown headings are rejected; approval stays with a person. |
| `tempa_decide` | Create a decision record; every required heading must be filled. |
| `tempa_log_change` | Append a changes entry (same rules as `tempa log`). |
| `tempa_check` | Run `tempa check` (drift, contract, task rules); `harness: true` also runs the `[harness]` commands. Findings as JSON. |
| `tempa_pr_body` | Pull request body for the current task. |

## Rules worth knowing

- Edit the source, not generated files. If you did edit a generated file, `tempa import --yes` absorbs it; `tempa sync --force` discards it.
- The pre-commit hook only syncs from staged sources, so a commit never contains generated output for work you have not staged.
- Plans are approved by a person (`tempa task approve`), never by an agent.
- Configure the gates in `tempa.toml`: `[harness]` commands (format, lint, test), protected paths, required decisions, maximum diff size. Generated files and dependency lockfiles (`Cargo.lock`, `bun.lock`, `package-lock.json`, ...) do not count toward the diff size.

## CI

Optional, and can come later. Local hooks can be skipped with `--no-verify`, so a CI step closes that gap. [`docs/tempa-devops.md`](crates/tempa-core/src/templates/devops.md) (written into your repo by `tempa init`) has ready-made Bitbucket Pipelines and GitHub Actions steps. Build the image with `docker build -t tempa:0.1.0 .`.

## Why tempa

Your team's agent setup, reviewed like code.

- **Edit anywhere, lose nothing.** A direct edit to a generated file is detected, and `tempa import --yes` moves it back into the source instead of overwriting it.
- **Honest about gaps.** `tempa doctor` shows, per agent, what it gets and what it cannot load natively, with the official docs behind every format in [docs/target-matrix.md](docs/target-matrix.md).
- **Gaps still reach the agent.** Commands or subagents an agent cannot load natively are served by the `tempa` MCP server (`tempa_context`).
- **Enforced in CI.** `tempa check` gates the contract itself: required headings, protected paths, harness commands, diff size, decision records for dependency changes.
- **One workflow, any agent.** An optional task flow (plan, decision, changes entry, pull request body) that looks the same whichever agent did the work.

## Development

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cargo build --release -p tempa-cli && scripts/e2e.sh target/release/tempa
```

This repo uses tempa on itself; see `AGENTS.md` and `HARNESS.md`. An experimental desktop app lives on the frozen [`desktop-app`](https://github.com/anggaralabshq/tempa/tree/desktop-app) branch; the CLI is the product. See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise, any contribution you submit for inclusion in tempa, as defined in the Apache-2.0 license, is dual licensed as above, without any additional terms or conditions.
