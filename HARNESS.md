# Harness

## Definition of done

- `tempa check` passes: contract documents, task rules, and every harness command in `tempa.toml`.
- A new test for every new behavior or fixed bug, shown to fail before the fix.
- `scripts/e2e.sh` passes for changes that touch a CLI flow.
- A `CHANGES.md` entry written with `tempa log`, naming the agent and model.

## Gates

- Pre-commit: `tempa sync --fix` (sync from staged sources, block on drift).
- Pre-push: `tempa check` (contract + harness: CLI format, lint, test; typecheck, Vitest, clippy and backend tests of the frozen `app/`). Needs Bun and the Tauri system dependencies.
- Task pull requests target the milestone branch and are gated by the hooks above.
- CI runs once per milestone, on the pull request into `main`: `.github/workflows/app-windows.yml` runs the app backend tests on Windows when `app/` or `tempa-core` changed. The reviewer runs `tempa check --base main` on that branch before approving.
