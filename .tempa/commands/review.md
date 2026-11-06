---
description: Review this branch's changes against the tempa contract
---
Review this branch's diff against `main`. Focus, if given: $ARGUMENTS

1. Read AGENTS.md (Conventions, Off-limits) and the active plan in docs/plans/.
2. For every behavior change, make sure a test would fail without it.
3. Look for data-loss risks: file writes, deletions, overwriting files the user owns.
4. Report findings as `path:line — problem — suggestion`, most severe first.
