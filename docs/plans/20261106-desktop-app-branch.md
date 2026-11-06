---
task_id: 20261106-desktop-app-branch
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Goal

Keep `main` a pure CLI repo: move the frozen desktop app to the `desktop-app` branch and replace its Windows-only CI with CI for the CLI.

## Steps

- Push branch `desktop-app` at the current `m4` (app intact), then delete `app/`, its third-party notice, ignore rules, workspace exclude and harness commands.
- Replace `.github/workflows/app-windows.yml` with `ci.yml`: fmt, clippy, tests on Linux/macOS/Windows and e2e on Linux/macOS, for pull requests into `main` only.
- Point README, AGENTS.md, HARNESS.md and CONTRIBUTING.md at the branch; translate tempa.toml comments.

## Files touched

app/ (deleted), THIRD_PARTY_NOTICES.md, .gitignore, Cargo.toml, tempa.toml, .github/workflows/{app-windows,ci}.yml, README.md, AGENTS.md, HARNESS.md, CONTRIBUTING.md

## Risks

The diff is ~7.7k deleted lines, over max_diff_lines; it is a pure removal of code archived on a branch, recorded in a decision. tempa.toml and the workflow are protected paths, changed with the owner's approval.
