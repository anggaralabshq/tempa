# Tempa M3-lite: `tempa mcp` — desain

Status: draf, menunggu persetujuan. Melengkapi spec utama (`2026-11-06-tempa-design.md`, bagian 7 dan 8).

## Kenapa sekarang, kenapa "lite"

- Pilot M1 dan riset pesaing (rulesync, Ruler) menunjukkan lapisan sync sudah komoditas. Pembeda tempa adalah **alur kerja dan kontrak yang sampai ke agent saat ia bekerja**.
- Spec utama menaruh Host MCP di M3 bersama ACP client dan aplikasi desktop (M2). Itu mahal dan menunggu M2.
- M3-lite memajukan bagian yang bisa berdiri sendiri: server MCP `tempa` yang dijalankan **oleh agent itu sendiri** lewat config MCP yang sudah di-generate `tempa sync`. Langsung bekerja di Claude Code, Codex, Gemini CLI, OpenCode, dan Cursor, tanpa aplikasi.
- Urutan milestone berubah: M1 → **M3-lite** → M2 → M3 (ACP, tool policy, worktree) → M4. **[call, perlu persetujuan]**

## Tujuan

1. Gap di `tempa doctor` (commands/agents yang tidak didukung Codex/Cursor, deny rules non-Claude) tertutup secara nyata: agent bisa mengambil isinya lewat `tempa_context`.
2. Agent menulis plan, decision, dan entri changes lewat tool yang **memvalidasi kontrak** — output seragam apa pun agent dan modelnya.
3. Agent bisa menjalankan `tempa check` dan menerima temuan terstruktur.

## Non-goal (tetap di M2/M3 penuh)

- Menolak edit file sebelum plan disetujui (tool policy). Tanpa host yang memegang sesi, ini tidak bisa ditegakkan; tetap ditegakkan oleh hook + `tempa check`.
- Preamble yang disuntik host, PlanPanel, worktree per task, ACP.
- Membuat PR otomatis (`tempa_handoff`). M3-lite hanya mengembalikan body PR.

## Cara kerja

- `tempa mcp`: server MCP via stdio (JSON-RPC 2.0, satu pesan per baris). Root proyek = direktori kerja (`git rev-parse --show-toplevel` bila di repo).
- Diimplementasikan tanpa dependency baru (serde_json yang sudah ada). Method: `initialize`, `notifications/initialized`, `ping`, `tools/list`, `tools/call`. Versi protokol: mengikuti yang diminta klien bila termasuk daftar yang didukung, selain itu versi terbaru yang didukung.
- `tempa sync` menambahkan server `tempa` (`command = "tempa"`, `args = ["mcp"]`) ke config MCP setiap agent yang aktif, kecuali `[mcp] serve = false` di `tempa.toml`. Server ini tidak masuk `.tempa/mcp.toml` (milik pengguna); ia ditambahkan saat render.
- `tempa doctor`: kolom `lifecycle` menjadi `mcp` bila server tempa dikonfigurasi untuk agent itu, `M3` bila tidak.
- Template `AGENTS.md` dari `tempa init` mendapat satu kalimat di bagian Konvensi: "Pakai tool MCP `tempa_*` untuk plan, decision, changes, dan check."

## Tool

| Tool | Input | Perilaku |
|---|---|---|
| `tempa_context` | `{ kind?: "all"\|"commands"\|"agents"\|"skills"\|"rules", name?: string }` | Mengembalikan konteks dari sumber netral: AGENTS.md, daftar/isi command, subagent, skill, protected paths, dan plan task aktif bila ada. Inilah penutup gap. |
| `tempa_status` | `{}` | Task aktif (dari branch), state plan, entri changes yang sudah ada, drift. |
| `tempa_plan` | `{ sections: { "<heading>": "<isi>" } }` | Mengisi/menimpa section plan task aktif. Menolak bila heading wajib (`[headings].plan`) kosong atau tidak dikenal. Tidak mengubah `state`/`approved_*`: approve tetap tindakan manusia (`tempa task approve`). |
| `tempa_decide` | `{ slug, sections }` | Membuat decision record dari template; menolak heading wajib kosong. |
| `tempa_log_change` | `{ summary, reason, decision?, harness, agent }` | Sama dengan `tempa log`, termasuk validasi satu baris. `agent` wajib berisi agent + model. |
| `tempa_check` | `{ contract_only?: bool }` | Menjalankan `tempa check` dan mengembalikan temuan terstruktur (code, severity, path, message). |
| `tempa_pr_body` | `{}` | Mengembalikan body PR (`tempa pr-body`). |

- Error validasi dikembalikan sebagai hasil tool dengan `isError: true` dan pesan perbaikan dalam Bahasa Indonesia, bukan error protokol, supaya agent bisa memperbaikinya sendiri.
- Tool tidak pernah menulis di luar root proyek dan tidak pernah menyentuh file hasil generate.

## Kriteria selesai

- Test protokol: initialize → tools/list → tools/call untuk setiap tool, lewat proses `tempa mcp` sungguhan.
- `tempa sync` menulis entri server `tempa` di kelima format MCP; `serve = false` menghapusnya.
- Pilot: Claude Code di repo ini memanggil `tempa_context` dan `tempa_status` (dibuktikan dengan `claude -p`). OpenCode dibuktikan lewat `opencode debug config` / `opencode mcp list` (kredensial model belum tersedia).

## Diserahkan ke builder

Skema JSON persis tiap tool, format teks `tempa_context`, dan pesan error.
