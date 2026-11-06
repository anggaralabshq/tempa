# Harness

## Definition of done

- `tempa check` passes: contract documents, task rules, and every harness command in `tempa.toml`.
- A new test for every new behavior or fixed bug, shown to fail before the fix.
- `scripts/e2e.sh` passes for changes that touch a CLI flow.
- A `CHANGES.md` entry written with `tempa log`, naming the agent and model.

## Gates

- Pre-commit: `tempa sync --fix` (sync from staged sources, block on drift).
- Pre-push: `tempa check` (contract + harness: format, lint, test).
- Task pull requests target the milestone branch and are gated by the hooks above.
- CI runs once per milestone, on the pull request into `main`: `.github/workflows/ci.yml` runs format, lint and tests on Linux, macOS and Windows, and the end-to-end script on Linux and macOS. Contract and task gates stay local, per task pull request.
