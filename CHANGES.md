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
