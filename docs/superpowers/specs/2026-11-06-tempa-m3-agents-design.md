# Tempa M3: agent dalam lifecycle — desain

Status: disetujui pemilik 2026-11-06 (ditulis dari garis besar spec utama, tanpa sesi tanya-jawab; tiga keputusan dikonfirmasi). Melengkapi `2026-11-06-tempa-design.md` bagian 4, 5.1, 7, 8, 11, 13.

Penanda: **[spec]** = sudah diputuskan di spec utama. **[user]** = keputusan pemilik untuk M3. **[call]** = keputusan penulis spec ini, boleh ditolak.

## Intent

M2 memberi terminal dan watcher: kontrak tetap sinkron, tapi agent yang dijalankan di terminal bekerja di luar lifecycle. Tidak ada yang mencegah agent mengedit sebelum plan disetujui, menyentuh protected path, atau mengaku "test lulus" tanpa bukti. M3 menutup G2: agent dijalankan oleh aplikasi lewat ACP, per task, di worktree sendiri, dengan host yang menjawab permission sesuai state lifecycle dan mencatat hasil harness sendiri.

Kriteria berhasil: engineer membuat task, memilih agent (Claude Code, Codex, Gemini CLI, OpenCode, atau Cursor) dan model, agent menulis plan lewat `tempa_plan`, engineer menekan Approve, agent mengimplementasi di worktree, `tempa_check` dijalankan host dan hasilnya tercatat, entri changes ditulis. Semua dengan alur yang sama untuk kelima agent.

## Tujuan

1. **Chat agent ACP per task [spec].** Pilih agent + model saat sesi dibuka. Pesan, tool call, dan diff tampil di chat. Batalkan giliran yang sedang jalan.
2. **Host MCP `tempa` disuntik ke setiap `session/new` [spec].** Tool sama untuk semua agent (`tempa_context`, `tempa_status`, `tempa_plan`, `tempa_decide`, `tempa_log_change`, `tempa_check`, `tempa_pr_body`).
3. **Permission prompt + tool policy [spec].** Host menjawab permission sesuai state; sisanya diteruskan ke engineer.
4. **Task + worktree [spec].** Membuat task = branch `tempa/<task-id>` + worktree sendiri + file plan. Daftar task per proyek di sidebar dengan badge state.
5. **Panel Task [spec].** State, preview plan + tombol Approve, hasil harness terakhir, PlanPanel (rencana ACP dari agent).
6. **Mode Eksplorasi [spec].** Chat tanpa task, di root proyek, dengan policy pra-Approved.
7. **Banner terminal [spec].** Agent v1 yang dijalankan langsung di tab terminal diberi banner "outside the tempa lifecycle".

## Non-goal M3 (tetap M4 atau sesudahnya)

- Review lintas-model, handoff PR (`tempa_handoff`), scorecard (M4) [spec]. Lifecycle M3 berakhir di **Record**; push dan PR tetap manual (`tempa pr-body`).
- Instalasi agent otomatis dari registry, signing, auto-update, e2e nightly dengan agent sungguhan (M4).
- GitHub Copilot sebagai agent v1 (setelah M4, sudah ada di registry ACP sebagai `github-copilot-cli`).
- "Jadikan task" dari percakapan Eksplorasi [call]: engineer membuat task baru dan menyalin konteks sendiri.

## Anti-goal

- Aplikasi punya aturan sendiri yang berbeda dari CLI/CI. Validasi dokumen tetap `tempa-core`; CI tetap penegak akhir [spec].
- Host memilih "allow always", mengganti mode agent, atau mengizinkan agent menulis protected path [spec].
- Hasil Verify berasal dari laporan agent. Hanya hasil harness yang dijalankan host yang dihitung [spec].
- Satu task merusak working copy engineer: agent task tidak pernah bekerja di checkout utama.

## Arsitektur [call]

```
app/src-tauri/src/
├─ acp/        # proses agent, koneksi ACP, sesi, policy
├─ tasks.rs    # task + worktree + state dari file plan
└─ main.rs     # `tempa-app --mcp <root>`: server MCP tempa (tempa_core::mcp) untuk sesi agent
```

- **Server MCP yang disuntik = binary aplikasi itu sendiri** dalam mode `--mcp` (`current_exe()`), memanggil `tempa_core::mcp`. Alasan: versi aturan = versi aplikasi (tidak bergantung `tempa` di PATH, temuan pilot M3-lite), dan proses itu bisa mencatat hasil `tempa_check` ke state host. Config MCP hasil `tempa sync` (untuk agent di terminal) tetap memakai CLI `tempa mcp`.
- **ACP ditulis di atas `serde_json` [call]**, hanya subset metode yang dipakai (seperti server MCP di M3-lite). Crate `agent-client-protocol` 3.x/5.x butuh executor async lokal non-Send (Termul sampai mem-vendor-nya); untuk satu klien dengan beberapa metode, thread + mpsc lebih sederhana. Butuh decision record. Modul ACP Termul (~83 ribu baris, terjalin dengan canvas/automation/akun) tidak disalin; polanya dipinjam seperti PTY di M2.
- **Peluncuran agent dari snapshot registry ACP** yang dibawa aplikasi (hanya 5 entri v1), dicek ulang saat plan ditulis:

  | Agent | Registry id | Cara jalan |
  |---|---|---|
  | Claude Code | `claude-acp` | `npx -y @agentclientprotocol/claude-agent-acp@<ver>` |
  | Codex | `codex-acp` | `npx -y @agentclientprotocol/codex-acp@<ver>` |
  | Gemini CLI | `gemini` | `npx -y @google/gemini-cli@<ver> --acp` |
  | OpenCode | `opencode` | `opencode acp` (binary di PATH) |
  | Cursor | `cursor` | `cursor-agent acp` (binary di PATH) |

  M3 tidak mengunduh apa pun: paket npx butuh `npx` di PATH; OpenCode/Cursor harus sudah terpasang. Bila tidak ada, picker menampilkan agent itu disabled dengan perintah instalasinya. PATH = PATH login shell (`env::terminal_path`, M2). Login/autentikasi agent tetap urusan CLI masing-masing; bila ACP mengembalikan `auth_required`, chat menampilkan instruksi login dari agent.
- **State host tanpa SQLite di M3 [user].** Sumber kebenaran state task tetap frontmatter plan di repo [spec]. Yang perlu disimpan host: hasil harness per task (`verify.json`) dan transkrip sesi (`<session>.jsonl`), di direktori data aplikasi per proyek. Alasan: dua jenis data, append-only dan dibaca utuh; SQLite + migrasi menyusul bersama scorecard di M4 bila memang perlu query. Menggantikan "SQLite (M3)" di spec M2.

## Lifecycle M3

```
Plan → Approved → Implement ⇄ Verify → Record
```

State disimpan di frontmatter plan (`state:`), ditulis oleh `tempa-core` [spec].

| Transisi | Syarat | Siapa |
|---|---|---|
| (buat) → Plan | judul task; worktree + branch `tempa/<task-id>` + plan kosong dibuat | engineer |
| Plan → Approved | plan lolos kontrak; **manusia** menekan Approve (`task_approve`, `approved_by` = `git config user.name`) | engineer |
| Approved → Implement | otomatis saat giliran agent pertama sesudah Approve | host |
| Implement → Verify | agent memanggil `tempa_check` (`harness: true`) atau engineer menekan Verify | agent/engineer |
| Verify → Record | semua perintah harness lulus, dijalankan oleh proses host | host |
| Verify → Implement | ada perintah harness gagal | host |
| Record | selesai bila entri changes untuk task ada; panel menampilkan perintah push + `tempa pr-body` | — |

[call] Field `state` baru selain `plan`/`approved`: `implement`, `verify`, `record`. CLI `tempa check` tidak memvalidasi nilai baru ini selain bahwa ia dikenal.

## Tool policy [spec + call]

Host menjawab `session/request_permission` dengan aturan berikut, berurutan:

1. Path di `protected_paths` (atau di luar worktree task / root proyek untuk Eksplorasi) → **tolak**, semua state [spec; batas worktree = call].
2. State sebelum Approved (dan Eksplorasi): kind `edit`, `delete`, `move` → **tolak** dengan pesan "Plan not approved yet: write the plan with tempa_plan and wait for approval." [spec].
3. `read`, `search`, `fetch`, `think` → izinkan sekali [spec].
4. `execute` dan kind lain → **tanya engineer** (Allow once / Reject) [spec].
5. Sesudah Approved: `edit`/`delete`/`move` di dalam worktree → izinkan sekali, tanpa prompt [user].

Host tidak pernah memilih opsi `allow_always`, dan tidak mengirim `session/set_mode` atas nama agent [spec]. Agent yang mengedit tanpa minta izin (mis. mode tertentu Claude Code) diluncurkan dengan mode yang selalu meminta izin; detail per agent dicatat di matriks agent. Penegakan akhir tetap CI [spec].

Preamble [spec]: giliran pertama tiap sesi diawali teks pendek dari host: id task, state, aturan lifecycle di atas dalam 5 baris, dan daftar tool `tempa_*`.

## Paritas lifecycle [spec]

Agent lulus paritas lifecycle bila, di sesi nyata: (a) menerima server MCP dari `session/new` dan memanggil tool `tempa_*`, (b) mengirim permission request sebelum edit. Hasil per agent dicatat di `docs/agent-matrix.md` (agent, versi adapter, a/b, catatan mode). Agent yang gagal diberi label "context-only" di picker dan `tempa doctor`. [call] Verifikasi ini manual oleh pemilik (butuh akun agent), satu kali per versi adapter; test otomatis memakai agent ACP palsu.

## Tampilan [call]

```
┌──────────────┬──────────────────────────────────────┬─────────────────────────┐
│ PROJECTS     │ [zsh] [💬 fix-login] [+]             │ [Task] [Contract]       │
│ ● tempa   2⚠ │ claude · sonnet           ⏹ Stop     │ fix-login   ● implement │
│   TASKS      │ ─────────────────────────────────── │ Plan  ✓ approved by ang │
│   ● fix-login│ you: fix the login redirect          │  1. ✓ reproduce         │
│     impl     │ agent: reading src/auth.ts …          │  2. ◐ patch redirect    │
│   ○ add-copi │ ┌ edit src/auth.ts  +4 −1 ─────────┐ │  3. ○ test              │
│     plan     │ │ allowed (approved)               │ │ Harness  ✗ test (2m ago)│
│   + New task │ └──────────────────────────────────┘ │  [Verify]               │
│   Explore    │ ┌ execute: bun test ───────────────┐ │ Worktree                │
│              │ │ [Allow once] [Reject]            │ │  ../tempa.worktrees/fix │
│ + Add project│ └──────────────────────────────────┘ │  [Open terminal here]   │
└──────────────┴──────────────────────────────────────┴─────────────────────────┘
```

- Chat adalah jenis tab di area kerja yang sama dengan terminal (tab, split pane M2 tetap berlaku).
- Panel kanan punya dua tab: Task dan Contract (M2).
- PlanPanel = langkah dari `SessionUpdate::Plan` agent, di panel Task di bawah plan.

## Perilaku yang ditentukan

- **Worktree [user]:** `<parent repo>/<nama repo>.worktrees/<task-id>`, dibuat `git worktree add -b tempa/<task-id>` dari branch aktif checkout utama. Hook dan `core.hooksPath` berlaku (worktree berbagi config). Watcher M2 juga dijalankan per worktree task yang terbuka. Menghapus task (dengan konfirmasi) = `git worktree remove` + hapus branch hanya bila sudah di-merge; selain itu worktree dibiarkan dan panel memberi perintahnya.
- **Task yang sudah ada:** branch `tempa/*` yang punya file plan tapi belum punya worktree tampil di daftar; membukanya membuat worktree dari branch itu (task dilanjutkan dari mesin lain [spec]).
- **Sesi:** satu sesi agent aktif per task. Menutup tab chat membatalkan giliran (`session/cancel`) lalu menghentikan proses agent; aplikasi ditutup → semua proses agent dihentikan (pola PTY M2). Sesi lama bisa dibaca dari transkrip; melanjutkan pakai `session/load` hanya bila agent mengiklankan `loadSession`, selain itu sesi baru dengan preamble.
- **`tempa_check` di sesi host:** menjalankan harness di worktree, menulis hasil (`{at, commands: [{name, ok, tail}]}`) ke `verify.json` task, dan menggerakkan state. Engineer menekan Verify = jalur yang sama.
- **Client capability ACP [call]:** aplikasi tidak mengiklankan `fs` maupun `terminal`; agent memakai filesystem dan shell miliknya, dan host hanya mengendalikan lewat permission. Lebih sedikit permukaan, sama untuk kelima agent.
- **Banner terminal [call]:** macOS/Linux saja di M3, dari nama proses foreground PTY (`claude`, `codex`, `gemini`, `opencode`, `cursor-agent`). Windows menyusul.
- **Bahasa:** semua teks UI dan pesan penolakan dalam Bahasa Inggris [spec].

## Kualitas dan test [call]

- Agent ACP palsu (binary test kecil di `app/src-tauri/tests/`) yang memutar skenario: plan → minta izin edit → panggil `tempa_*` → selesai. Dipakai untuk test koneksi, policy, preamble, dan lifecycle end-to-end tanpa akun.
- Unit test: tabel policy (state × kind × path), transisi state, path worktree, parsing snapshot registry.
- Frontend: Vitest untuk reducer chat (urutan update, tool call yang diperbarui), panel Task, picker.
- Windows CI (workflow M2) ikut menjalankan test backend, termasuk agent palsu lewat pipa stdio dan resolusi `npx.cmd`.
- Uji manual: pemilik mengisi `docs/agent-matrix.md` untuk kelima agent; anggota tim Windows mengulang untuk satu agent.

## Gate diff dan urutan PR [call]

Masing-masing ≤ 800 baris, satu task plan:

1. `tempa-app --mcp` + state `implement/verify/record` di `tempa-core` + pencatatan `verify.json`.
2. Koneksi ACP: proses agent, initialize, `session/new` dengan MCP disuntik, prompt, update, cancel; agent palsu.
3. Tab chat: pesan, tool call, picker agent/model dari snapshot registry, status agent tidak terpasang.
4. Tool policy + permission prompt + preamble.
5. Task: buat/lanjut/hapus, worktree, daftar di sidebar, panel Task + Approve + Verify.
6. Lifecycle end-to-end + PlanPanel + mode Eksplorasi.
7. Banner terminal + `docs/agent-matrix.md` + label context-only di picker/doctor.

Review akhir serius (reviewer baru) setelah PR 7.

## Diserahkan ke builder

Nama tipe dan command Tauri, struktur komponen React, skema persis `verify.json` dan transkrip, teks preamble, cara tiap adapter menerima pilihan model (`session/set_model` atau config option), visual.
