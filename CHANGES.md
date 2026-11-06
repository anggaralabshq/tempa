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
