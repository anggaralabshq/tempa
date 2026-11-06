# Matriks target agent (diverifikasi 2026-11-06)

Dipakai oleh `tempa sync` (`crates/tempa-core/src/targets.rs`). Perbarui file ini dan kodenya bersamaan bila dokumentasi agent berubah.

| Sumber | Claude Code | Codex | Gemini CLI | OpenCode | Cursor |
|---|---|---|---|---|---|
| context | `CLAUDE.md` = header + `@AGENTS.md` | native `AGENTS.md` | `.gemini/settings.json` → `context.fileName = ["AGENTS.md"]` | native `AGENTS.md` | `.cursor/rules/tempa.mdc` (`alwaysApply: true`) |
| commands | `.claude/commands/<rel>` (md + frontmatter, `$ARGUMENTS`) | **gap** — custom prompts deprecated, hanya user-level `~/.codex/prompts` | `.gemini/commands/<rel>.toml` (`description`, `prompt`, `{{args}}`; subfolder → `:`) | `.opencode/commands/<rel>` (md + `description`, `$ARGUMENTS`) | `.cursor/commands/<rel>` (md tanpa frontmatter) — *belum terkonfirmasi docs, lihat catatan* |
| agents | `.claude/agents/<rel>` | **gap** | `.gemini/agents/<rel>` (frontmatter `name`, `description`) | `.opencode/agents/<rel>` (frontmatter `description`, `mode: subagent`) | **gap** |
| skills | `.claude/skills/<name>/` | `.agents/skills/<name>/` | `.agents/skills/<name>/` (alias resmi `.gemini/skills`) | `.agents/skills/<name>/` | `.agents/skills/<name>/` |
| mcp | `.mcp.json` `mcpServers` | `.codex/config.toml` `[mcp_servers.<name>]` (`command`, `args`, `env` / `url`; hanya proyek trusted) | `.gemini/settings.json` `mcpServers` | `opencode.json` `mcp` (`type: "local"`, `command: [..]`, `environment` / `type: "remote"`, `url`) | `.cursor/mcp.json` `mcpServers` |
| settings-deny | `.claude/settings.json` `permissions.deny` `Edit(/<path>)` | **gap** | **gap** | **gap** | **gap** |

## Catatan

- `.agents/skills/` dibaca bersama oleh Codex, Gemini, OpenCode, dan Cursor, jadi cukup ditulis sekali. OpenCode dan Cursor juga membaca `.claude/skills/`, sehingga skill yang sama terlihat dari dua lokasi dengan isi identik. Diterima: isinya dijamin sama oleh `tempa sync`.
- Claude: `Edit(...)` mencakup semua tool penulis file (Write, MultiEdit). `/path` di project settings relatif ke root proyek; dipilih agar konsisten dengan glob `protected_paths` di CI yang juga relatif ke root.
- Gemini CLI: untuk pengguna tier gratis/Google One, Gemini CLI diganti Antigravity CLI sejak 2026-06-18. Target ini tetap berlaku untuk pengguna Gemini CLI berbayar/enterprise; evaluasi ulang target Antigravity di luar M1.
- Cursor commands: halaman docs commands mengarah ke Skills; Cursor menyediakan migrasi command → skill (`disable-model-invocation: true`). `.cursor/commands/` dipertahankan di M1; pindahkan ke skill bila Cursor berhenti membacanya.
- MCP env/header: sumber memakai sintaks Claude `${VAR}`. OpenCode → `{env:VAR}`. Codex meneruskan `env` dan `http_headers` apa adanya: `KEY = "${KEY}"` → `env_vars = ["KEY"]`, header `"${VAR}"` → `env_http_headers`, `Authorization: "Bearer ${VAR}"` → `bearer_token_env_var`; bentuk lain (mis. `DB = "${DB_URL}"`, ganti nama) dilaporkan sebagai gap `mcp`. Cursor/Gemini: `${VAR}` diteruskan apa adanya (belum terverifikasi docs untuk Cursor).
- Cursor membaca `AGENTS.md` menurut sumber pihak ketiga, tapi belum terkonfirmasi docs resmi; `.cursor/rules/tempa.mdc` tetap di-generate.
- Server `tempa` (M3-lite): `tempa sync` menambahkan `tempa` (`command = "tempa"`, `args = ["mcp"]`) ke kelima config MCP di atas saat render, kecuali `[mcp] serve = false` atau `.tempa/mcp.toml` sudah punya server bernama `tempa`. Tidak ditulis ke `.tempa/mcp.toml`, dan `tempa import` tidak menyerapnya. Claude: `.claude/settings.json` mendapat `enabledMcpjsonServers: ["tempa"]` supaya server proyek tidak perlu disetujui manual. Codex hanya memuat `.codex/config.toml` di proyek trusted. Command/subagent yang tidak didukung agent (gap di `tempa doctor`) bisa diambil lewat `tempa_context`.

## Sumber

- Codex: https://learn.chatgpt.com/docs/build-skills, https://learn.chatgpt.com/docs/custom-prompts, https://learn.chatgpt.com/docs/config-file/config-reference
- Gemini CLI: https://geminicli.com/docs/cli/skills, https://geminicli.com/docs/cli/custom-commands, https://geminicli.com/docs/core/subagents/, https://geminicli.com/docs/cli/gemini-md
- OpenCode: https://opencode.ai/docs/commands, https://opencode.ai/docs/agents, https://opencode.ai/docs/skills, https://opencode.ai/docs/mcp-servers
- Cursor: https://cursor.com/docs/context/skills
- Claude Code: https://code.claude.com/docs/en/settings, https://code.claude.com/docs/en/permissions
