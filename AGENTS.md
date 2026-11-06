# tempa

Rust CLI that keeps one context contract (AGENTS.md + `.tempa/` + `tempa.toml`) and generates the files Claude Code, Codex, Gemini CLI, OpenCode, Cursor and GitHub Copilot read.

## Stack

- Rust edition 2024, MSRV 1.85 (CI image `rust:1.85-alpine`); local toolchain stable.
- Workspace: `crates/tempa-core` (library, all logic) and `crates/tempa-cli` (binary `tempa`, clap, thin).
- Git is called through the `git` CLI (`crates/tempa-core/src/gitx.rs`), not libgit2.
- `app/` is a Tauri desktop app, frozen: keep its tests green, add no features.

## Commands

- Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Format: `cargo fmt --all` (check: `cargo fmt --check`)
- E2E: `cargo build --release -p tempa-cli && scripts/e2e.sh target/release/tempa`

## Conventions

- TDD: failing test first, then code. Regressions found in review go to `crates/tempa-core/tests/review_fixes.rs` or the module's test file.
- Logic in `tempa-core` is pure functions where possible; the CLI only parses arguments and prints.
- All text in the repo and for users (CLI messages, help, README, templates, docs) is English.
- Every finding is a `Finding { code, severity, path, message }`; codes are stable, e.g. `drift.modified`.
- Never overwrite a user's file without proof tempa wrote it (lock hash) or `--force`.
- A target format changes only together with `docs/target-matrix.md` and its official docs source.

## Off-limits

- `crates/tempa-core/tests/fixtures/` changes only when a test needs new data.
- Do not edit generated files (`CLAUDE.md`, `.claude/`, `.gemini/`, `.cursor/`, `.opencode/`, `.agents/`, `.github/agents/`, `.github/skills/`, `.github/mcp.json`, `opencode.json`, `.codex/`); change the source, then `tempa sync`.
