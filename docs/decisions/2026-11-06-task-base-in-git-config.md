## Context

tempa check took its base from --base, the CI PR destination, or [ci].base. Task branches started from a milestone branch (m4) were measured against main, so the pre-push hook failed the diff and one-plan gates on work already counted elsewhere.

## Options

1. A TEMPA_BASE env var. 2. A base field in the plan frontmatter. 3. Repo-local git config written by tempa task new (branch.<name>.tempaBase), read by gitx::base_name.

## Decision

Option 3: no new file format, no extra step for the user, and git already scopes it per branch. gitx::base_name gains a root parameter (public API change).

## Consequences

Another clone of the same task branch falls back to [ci].base; pass --base there. CI keeps using the PR destination.

## Agent + model

claude-code / claude-opus-5-5
