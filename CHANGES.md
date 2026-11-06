# Changes

## 2026-11-06 · 20261106-english-vocabulary
- Summary: English default contract vocabulary with legacy Indonesian aliases
- Reason: CLI is going public; all other user-facing text is English
- Decision: docs/decisions/2026-11-06-english-default-vocabulary.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-english-docs
- Summary: Move this repo's contract and docs to English
- Reason: English is the default vocabulary and the repo is going public
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-public-readiness
- Summary: Add CONTRIBUTING, SECURITY and README positioning
- Reason: Public readiness before the first release
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-desktop-app-branch
- Summary: Move the frozen desktop app to branch desktop-app; CI for the CLI
- Reason: main becomes a CLI-only repo
- Decision: docs/decisions/2026-11-06-desktop-app-to-branch.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-fix-homebrew-checksums
- Summary: Homebrew formula reads the uploaded .sha256 names and fails on bad checksums
- Reason: Review: the first release would have pushed a formula with empty sha256 values
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-harness-git-env
- Summary: Harness commands no longer inherit the hook's GIT_DIR and related variables
- Reason: Pushing from a worktree let the test suite's temp-repo git calls write into the pushing repo (bare flag, user config, branch commit)
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-review-fixes-small
- Summary: Fix small m4 review findings (TOML escaping, Codex description, Cursor agents, task base, pr-body base)
- Reason: Fresh review of m4 before the milestone merge
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-mcp-import-fixes
- Summary: Import absorbs .github/mcp.json servers; Cursor MCP variables round-trip
- Reason: m4 review: user MCP servers could be dropped or broken by sync
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-absorb-new-agent-dirs
- Summary: Import absorbs hand-written Copilot and Codex agents; clear advice for .github/skills
- Reason: m4 review: files in the new tempa-owned dirs failed tempa check with no way out
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-name-collisions
- Summary: Report agent and Copilot skill name collisions instead of dropping items
- Reason: m4 review: same-named items in subfolders silently overwrote each other
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-archive-old-docs
- Summary: Archive internal and Indonesian docs; English decision headings
- Reason: Public repo shows the docs a reader needs; history keeps the rest
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-readme-demo
- Summary: README demo GIF (vhs tape) and aligned doctor table
- Reason: Public readiness: show the product on the first screen
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-ci-portability
- Summary: Make e2e, harness tests and CI portable across Linux, macOS and Windows
- Reason: First milestone CI run failed: e2e on master-default runners, sh-only harness tests on Windows
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-windows-clippy
- Summary: Unix-only harness env test compiles away on Windows
- Reason: Milestone CI: unused imports failed clippy on Windows
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-lf-checkout
- Summary: Force LF checkouts so templates and fixtures match on Windows
- Reason: Milestone CI: autocrlf checkouts produced CRLF plan templates and fixtures
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-release-version-check
- Summary: Release stops early when the tag does not match the crate version
- Reason: m4 review: a mismatched tag would ship binaries reporting the wrong version and fail crates.io
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-readme-after-release
- Summary: README install section after the v0.1.0 release
- Reason: v0.1.0 is on Homebrew, crates.io and GitHub Releases
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-readme-why-tempa
- Summary: README: Why tempa section instead of a competitor comparison
- Reason: Owner: the README should not send readers to other tools
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5
