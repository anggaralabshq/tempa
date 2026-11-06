# Changes

## 2026-11-06 · 20261106-adopsi-tempa
- Ringkasan: Dogfooding tempa di repo sendiri
- Alasan: Pilot M1: membuktikan kontrak terbaca agent asli dan alur hook berjalan
- Decision: -
- Harness: lulus
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-backlog-pilot-minor
- Ringkasan: Temuan pilot dan minor M1 ditutup
- Alasan: Pilot dogfooding menemukan bug pr-body dan default yang terikat Bitbucket; minor review akhir ditunda dari M1
- Decision: -
- Harness: lulus
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-english-messages
- Ringkasan: English for user-facing text
- Alasan: Repo and tool are published in English
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-mcp-protocol
- Ringkasan: tempa mcp server: protocol core and read tools
- Alasan: Agents need the contract over MCP (M3-lite, part 1 of 3)
- Decision: docs/decisions/2026-11-06-mcp-server-on-serde-json.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-mcp-write-tools
- Ringkasan: tempa mcp write tools: plan, decision, changes entry
- Alasan: Uniform workflow documents from every agent (M3-lite, part 2 of 3)
- Decision: docs/decisions/2026-11-06-mcp-server-on-serde-json.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-m3-lite-mcp
- Ringkasan: Register the tempa MCP server in every agent config
- Alasan: Every agent starts tempa mcp itself (M3-lite, part 3 of 3)
- Decision: docs/decisions/2026-11-06-mcp-server-on-serde-json.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-lockfiles-not-counted
- Ringkasan: Lockfiles do not count toward the diff gate
- Alasan: Generated, not reviewed; the M2 app scaffold would fail the gate on lockfiles alone
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-app-scaffold
- Ringkasan: Desktop app scaffold (Tauri 2, React, Bun)
- Alasan: M2 shell: separate workspace so the CLI never builds Tauri
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-app-terminal
- Ringkasan: Terminal: pty per pane with the login-shell PATH
- Alasan: M2: agents run in the app's terminal and must find tempa, claude, codex
- Decision: docs/decisions/2026-11-06-desktop-stack-tauri.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-app-tabs-panes
- Ringkasan: Tabs and split panes
- Alasan: M2: several agents and shells side by side per project
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-app-projects
- Ringkasan: Projects: add, remove, switch without losing terminals
- Alasan: M2: one window for several repos
- Decision: docs/decisions/2026-11-06-folder-picker-dialog-plugin.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5
## 2026-11-06 · 20261106-app-harness-ci
- Ringkasan: App checks in the harness; Windows backend tests in CI
- Alasan: Owner decision: one gate for CLI and app; Windows must work in M2
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-app-contract-panel
- Ringkasan: Contract panel: findings, drift actions, agents, hooks, CLI version
- Alasan: M2: keep the contract visible and fixable without the CLI
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-app-watcher
- Ringkasan: Watcher: source edits sync automatically, drift shows as a sidebar badge
- Alasan: M2: keep agent files in sync without remembering to run tempa sync
- Decision: docs/decisions/2026-11-06-watcher-on-notify.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-m2-review-fixes
- Ringkasan: M2 review fixes: safe destructive actions, missing folders, Windows consoles, watcher and paste stalls
- Alasan: Important findings from the fresh M2 final review
- Decision: docs/decisions/2026-11-06-core-command-and-absorbed.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-m3-host-mcp-verify
- Ringkasan: Host MCP mode in the app binary; Verify recorded by the host moves the task state
- Alasan: M3: harness results must come from the host, not the agent
- Decision: docs/decisions/2026-11-06-host-mcp-in-app-binary.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-m3-acp-client
- Ringkasan: ACP client: agent process, JSON-RPC both ways, session with the tempa MCP server
- Alasan: M3: run the five agents over ACP inside the lifecycle
- Decision: docs/decisions/2026-11-06-acp-client-on-serde-json.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-fix-hookless-test-repos
- Ringkasan: Test repos commit without hooks, so app tests pass without the tempa CLI (Windows CI)
- Alasan: PR #16 and #17 were merged with red Windows CI
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-m3-agents-chat
- Ringkasan: Agent table and chat sessions for the UI (start, prompt, cancel, stop, model, permissions)
- Alasan: M3: run the five agents from the app
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-freeze-m3-wiring
- Ringkasan: Freeze M3 (1/3): CI once per milestone; app back to M2 wiring, chat, agents, host MCP and verify removed
- Alasan: owner refocused tempa on contract sync; M3 lives on branch m3
- Decision: docs/decisions/2026-11-06-refocus-on-contract-sync.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-freeze-m3-acp
- Ringkasan: Freeze M3 (2/3): ACP client connection, session and tests removed
- Alasan: owner refocused tempa on contract sync; M3 lives on branch m3
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-freeze-m3-core
- Ringkasan: Freeze M3 (3/3): fake ACP agent and core task::set_state removed; app and core back to M2
- Alasan: owner refocused tempa on contract sync; M3 lives on branch m3
- Decision: docs/decisions/2026-11-06-refocus-on-contract-sync.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-check-task-base
- Ringkasan: tempa check measures a task against the branch it started from; doctor says off instead of M3; repo hooks in English
- Alasan: milestone branches (m4) failed the pre-push gates against main
- Decision: docs/decisions/2026-11-06-task-base-in-git-config.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-check-task-base
- Ringkasan: tempa check measures a task against the branch it started from; doctor says off instead of M3; repo hooks in English
- Alasan: milestone branches (m4) failed the pre-push gates against main
- Decision: docs/decisions/2026-11-06-task-base-in-git-config.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-cursor-native
- Ringkasan: Cursor target follows current docs: AGENTS.md native (no tempa.mdc), MCP ${env:VAR}, .cursor/agents, CLI deny in .cursor/cli.json
- Alasan: m4 format verification against official Cursor docs
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-codex-agents-opencode-deny
- Ringkasan: Codex subagents in .codex/agents/*.toml; OpenCode denies protected edits via permission.edit
- Alasan: m4 format verification: both now have an official project-level option
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-copilot-target
- Ringkasan: GitHub Copilot is the sixth target: AGENTS.md native, commands as user-invoked skills in .github/skills, .github/agents, .github/mcp.json
- Alasan: a teammate uses Copilot; m4 priority 1
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-dogfood-copilot
- Ringkasan: tempa's own repo enables the Copilot target
- Alasan: dogfood the sixth target
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-license-and-crate-metadata
- Ringkasan: Dual MIT OR Apache-2.0 license; crates.io metadata for tempa-cli and tempa-core
- Alasan: prepare release and distribution
- Decision: docs/decisions/2026-11-06-publish-as-tempa-cli.md
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-release-workflow
- Ringkasan: Release workflow on version tags: binaries for 5 targets with checksums, crates.io publish, Homebrew formula; releasing guide
- Alasan: release and distribution for the public launch
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

## 2026-11-06 · 20261106-pin-release-actions
- Ringkasan: Release workflow: actions pinned to commit SHAs, per-job permissions, publish secrets scoped to their steps
- Alasan: security review: the workflow holds publish tokens
- Decision: -
- Harness: pass
- Agent: claude-code / claude-opus-5-5

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
