## Context

tempa is getting ready for a public release. The owner chose a dual MIT OR Apache-2.0 license and publishing on crates.io. The crate name tempa is already taken there (an unrelated animation-time crate).

## Options

1. Publish as tempa-cli (binary tempa) plus tempa-core. 2. Pick a new product name. 3. Skip crates.io (GitHub Releases, Homebrew, cargo install --git only).

## Decision

Option 1 (owner). tempa-core gets a version on its path dependency so both crates publish; keywords, categories, repository and descriptions are set in the workspace.

## Consequences

Users run cargo install tempa-cli and get the tempa binary. tempa-core is a public library on crates.io, so breaking its API needs a semver bump. Publishing needs a CARGO_REGISTRY_TOKEN secret (release workflow).

## Agent + model

claude-code / claude-opus-5-5
