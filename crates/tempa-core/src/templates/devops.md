# Tempa: CI setup (one time)

Local git hooks already run `tempa sync --fix` (pre-commit) and `tempa check` (pre-push). Hooks can be skipped with `--no-verify`, so enforce the contract in CI once you are ready.

## 1. CI step

The step only checks drift and the contract documents. It needs no secrets and runs no tests; keep lint/test in your existing steps. It needs the full git history (`tempa check` diffs against the base branch).

Bitbucket Pipelines:

```yaml
pipelines:
  pull-requests:
    '**':
      - step:
          name: tempa contract
          image: <registry>/tempa:{version}
          clone:
            depth: full
          script:
            - tempa check --contract-only
```

GitHub Actions:

```yaml
on: pull_request
jobs:
  tempa:
    runs-on: ubuntu-latest
    container: <registry>/tempa:{version}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - run: git config --global --add safe.directory "$GITHUB_WORKSPACE"
      - run: tempa check --contract-only --base "origin/${{ github.base_ref }}"
```

Set `<registry>` once the image hosting is decided. The image version must match `tempa_version` in `tempa.toml`.

## 2. Merge rules

- Require the tempa step to pass (Bitbucket: Minimum successful builds 1; GitHub: required status check).
- Require at least one approval.
- Bitbucket Premium: tick "Prevent a merge with unresolved merge checks".

## 3. Don't

- Don't accept a build status reported from a developer machine in place of this step.
