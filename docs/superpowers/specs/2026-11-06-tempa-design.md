# Tempa — Dokumen Desain

Tanggal: 2026-11-06
Status: draft, menunggu persetujuan
Riset pendukung: https://claude.ai/artifact/SizY7VoXYCYupJueVAu88X

Penanda: **[user]** = diputuskan pemilik proyek. **[call]** = keputusan penulis desain, boleh ditolak. Sisanya diserahkan ke builder (lihat bagian 14).

---

## 1. Intent

Dalam satu proyek, engineer memakai coding agent dan model berbeda. Engineer yang dominan memakai Claude Code menumpuk pengetahuan proyek di format khusus Claude (`CLAUDE.md`, `.claude/commands/`, `.claude/agents/`, `.claude/skills/`, hooks di `.claude/settings.json`). Agent lain (Codex, Gemini CLI, Cursor, OpenCode) tidak membaca format itu, sehingga engineer non-Claude susah catch up dan output antar-engineer tidak konsisten. **[user]**

Tempa adalah terminal desktop tim tempat semua coding agent berjalan dengan **konteks yang sama, tool yang sama, dan gate output yang sama**, ditambah CLI headless yang menegakkan aturan yang sama di CI.

Tesis: jangan menyeragamkan agent. Seragamkan (1) konteks yang masuk, (2) tool selama sesi, (3) gate yang harus dilewati output. Semuanya di atas standar terbuka: AGENTS.md, ACP, MCP, git.

## 2. Tujuan, non-goal, anti-goal

### Tujuan
- **G1 — Paritas konteks hari pertama [user].** Engineer non-Claude mendapat konteks setara dengan engineer Claude Code sejak hari pertama, lewat satu kali `tempa import`.
- **G2 — Siklus development lengkap [user].** Setiap task melewati alur yang sama apa pun agent-nya: Intake → Plan → Approve → Implement → Verify → Record → Review → Handoff (PR). Tidak ada langkah terlewat, setiap task bisa diaudit.
- **G3 — Produk siap produksi [user].** Aplikasi desktop yang solid, bukan prototipe.

### Anti-goal (kegagalan walaupun fitur "jalan") [user]
- **A1 — Engineer non-Claude tetap ketinggalan konteks**, termasuk karena drift: konteks baru ditulis dalam format Claude setelah import dan tidak mengalir ke agent lain.
- **A2 — Output tidak konsisten** antar-agent/model.

### Non-goal v1 [call]
- SSH/SFTP, embedded browser, canvas, akses mobile/remote, server headless multi-user.
- Code editor bawaan. Engineer pakai editor masing-masing; Tempa cukup punya preview markdown untuk approve plan.
- Menjamin setiap task sukses di percobaan pertama. "Sempurna" = tidak ada langkah terlewat dan hasil bisa diaudit.
- Dashboard tim terpusat / telemetri jarak jauh.
- Menyeragamkan model atau melarang agent tertentu.

## 3. Kriteria sukses (bisa dicek)

1. Di repo contoh dengan `CLAUDE.md` + `.claude/{commands,agents,skills}` + `.claude/settings.json`, `tempa import && tempa sync` menghasilkan file target untuk 5 agent v1, dan `tempa doctor` melaporkan per agent: item yang terbaca, item yang ditegakkan host, item gap. Tidak ada item yang hilang tanpa dilaporkan.
2. Edit langsung ke `CLAUDE.md` atau file baru di `.claude/commands/` setelah import terdeteksi oleh aplikasi (ditawarkan "serap") dan membuat `tempa check` gagal di CI sampai diserap.
3. Task yang sama dijalankan dengan Claude Code dan Codex menghasilkan PR dengan struktur identik (body PR, file plan, entri changes, hasil harness) dan keduanya lolos `tempa check`.
4. Di dalam aplikasi, permission edit/delete/move dari agent ditolak host sebelum plan disetujui, dan permission ke `protected_paths` selalu ditolak. Ini **best effort [user]**: edit yang lolos lewat jalur lain (shell, mode agent) tidak dijamin tercegah, tapi tertangkap di gate CI dan review merge.
5. Engineer yang tidak memakai aplikasi Tempa **boleh** menyelesaikan task lewat CLI, dan PR-nya lolos CI bila memenuhi kontrak yang sama persis dengan PR dari aplikasi **[user]**. PR yang tidak memenuhi kontrak gagal di `tempa check`, dari mana pun asalnya.
6. Tidak ada PR yang merge tanpa approval review dari manusia di Bitbucket (merge check, dipaksa) **[user]**.
7. Pelanggaran kontrak tertangkap **sebelum** CI di mesin engineer (gate Handoff di aplikasi, hook pre-push), sehingga pipeline milik DevOps jarang menjadi tempat pertama kegagalan kontrak terlihat **[user]**.

### 3.1 Batas kepercayaan [user]

Penegakan berlapis, dari yang paling awal **[user: Bitbucket Cloud Premium; CI dikelola DevOps]**:

| Lapis | Di mana | Siapa yang memasang | Bisa dilewati? | Fungsi |
|---|---|---|---|---|
| L1 | Aplikasi: tool `tempa_*` menolak dokumen yang tidak lolos kontrak; Handoff hanya bisa setelah `tempa check` lulus | otomatis | tidak dari dalam app | umpan balik tercepat |
| L2 | Git hook: pre-commit `tempa sync --fix` (sync sumber → target otomatis + stage; blokir bila target diedit langsung), pre-push `tempa check` | `tempa init` / aplikasi saat proyek dibuka (set `core.hooksPath` ke `.tempa/hooks/`, di-commit di repo) | ya (`--no-verify`) | menangkap engineer di luar app sebelum push |
| L3 | Bitbucket Pipelines: satu step `tempa check --contract-only` | **DevOps, sekali saja** | tidak | penegakan untuk semua PR |
| L4 | Merge check Bitbucket (Premium, "Prevent a merge with unresolved merge checks"): minimum successful builds ≥ 1 + minimum approvals ≥ 1 dari manusia | DevOps / admin repo, sekali saja | tidak | gerbang yang tidak bisa dipalsukan agent |

Prinsip L3 agar DevOps tidak terbebani **[call]**:
- `tempa check --contract-only` hanya memeriksa drift + kontrak dokumen, tanpa menjalankan harness. Cepat (detik), tanpa dependensi bahasa proyek, tanpa secret. Lint/test tetap di step pipeline yang sudah dimiliki DevOps, jadi tidak ada duplikasi.
- Logika aturan ada di `tempa.toml` di repo, bukan di pipeline. Mengubah aturan tidak perlu menyentuh pipeline DevOps.
- `tempa init` menghasilkan `docs/tempa-devops.md`: snippet step pipeline siap tempel + daftar pengaturan merge check, untuk diserahkan ke DevOps.
- Tidak dipakai **[call]**: memposting build status "sukses" ke Bitbucket dari mesin lokal. Status semacam itu bisa dipalsukan siapa saja yang memegang token, termasuk agent.
- Approve plan di aplikasi dan penanda `approved_by` di frontmatter plan adalah **panduan proses**, bukan bukti keamanan. CI hanya memeriksa strukturnya.
- Gate di aplikasi (tool policy) bersifat best effort; tujuannya mengarahkan agent ke alur yang benar, bukan menjadi sandbox.

## 4. Batasan

- Platform: macOS, Windows, Linux. **[call]**
- Git host v1: **Bitbucket Cloud, paket Premium [user]**. CI (Bitbucket Pipelines) dikelola tim DevOps **[user]**.
- Pembuatan PR v1: push branch, buka URL "create pull request" Bitbucket dengan source/destination terisi, dan salin body PR ke clipboard. Pembuatan PR lewat REST API (butuh API token per engineer) menyusul setelah 1.0. **[call]**
- Agent v1 yang dijamin paritas: Claude Code, Codex, Gemini CLI, OpenCode, Cursor. Agent ACP lain boleh jalan, tanpa jaminan. **[call]**
- Paritas punya dua tingkat **[call]**:
  - **Paritas konteks** (wajib untuk kelima agent, M1): file target ter-generate + semua gap tertutup lewat `tempa_context` atau dilaporkan `tempa doctor`.
  - **Paritas lifecycle** (target M3): agent menerima host MCP yang disuntikkan, mengirim permission request, dan memakai tool `tempa_*`. Agent yang setelah verifikasi tidak memenuhi syarat ini (misal karena ekstensi ACP non-standar) diberi label "context-only" di `tempa doctor` dan UI; engineer tetap bisa memakainya, dan kontrak tetap ditegakkan oleh CI.
- Lisensi distribusi Tempa, sertifikat signing (Apple Developer ID, code signing Windows), dan hosting artefak update **belum diputuskan**; ini prasyarat M4, tidak mengubah arsitektur.
- Repo baru dari nol; modul matang dari Termul (MIT) boleh disalin dan diadaptasi dengan mempertahankan notice lisensi. **[user]**
- Tidak ada data proyek yang keluar dari mesin engineer kecuali lewat agent yang dipilih engineer sendiri. **[call]**

## 5. Arsitektur

```
tempa/
├─ crates/
│  ├─ tempa-core/     # semua logika: config, importer, compiler, contract, gate, harness, lifecycle
│  └─ tempa-cli/      # binary `tempa` (headless): init, import, sync, check, doctor
├─ app/
│  ├─ src-tauri/      # shell desktop: PTY, ACP client, host MCP, watcher; memanggil tempa-core
│  └─ src/            # UI React + TypeScript
└─ docs/
```

- Stack: Tauri 2, Rust, React + TypeScript, Bun — sama dengan Termul agar modul yang dipinjam bisa dipindah tanpa tulis ulang. **[call]**
- **Satu sumber logika:** desktop dan CLI memanggil `tempa-core` yang sama, sehingga aturan di aplikasi dan di CI tidak bisa berbeda. **[call]**

### 5.1 Modul yang dipinjam dari Termul [user: pinjam; call: daftar]

Referensi: https://github.com/gnoviawan/termul (v0.4.52). Salin, adaptasi, catat di `THIRD_PARTY_NOTICES.md`.

| Kebutuhan | Sumber Termul |
|---|---|
| PTY & terminal | `src-tauri/src/pty/`, `trackers/` |
| ACP client, proses agent, registry, install | `src-tauri/src/acp/{client,agent_process,catalog,install,manager,session}`, `acp_registry_snapshot*`, `agent_registry*` |
| Host MCP yang disuntik ke tiap `session/new` | `src-tauri/src/acp/host_mcp/{mod,parent,child,bridge}.rs` |
| Jawaban permission oleh host | `src-tauri/src/acp/session/tool_policy.rs` |
| Guard operasi destruktif | `src-tauri/src/acp/destructive_guard.rs` |
| Worktree | `src-tauri/src/worktree/` |
| Penulisan file atomik | `src-tauri/src/acp/atomic_file*` |
| Plan panel ACP | `acp/client.rs` (`SessionUpdate::Plan`), `PlanPanel` di renderer |

Tidak dibawa: `ssh/`, `canvas/`, `browser_*`, `remote/`, `web/`, `server_*`, `automation/` (v1 cukup lifecycle sendiri; lihat 8), editor. **[call]**

## 6. Lapisan Konteks (G1, A1)

### 6.1 Sumber kanonik [call]

```
AGENTS.md                 # instruksi proyek, diedit manusia (standar terbuka, dibaca native banyak agent)
tempa.toml                # konfigurasi Tempa
.tempa/
  commands/<name>.md      # prompt template; frontmatter: description, arguments
  agents/<name>.md        # definisi subagent; frontmatter: name, description, tools; body: prompt
  skills/<name>/SKILL.md  # format Agent Skills (SKILL.md + file pendukung)
  lock.json               # hash file hasil generate, untuk deteksi drift
```

`AGENTS.md` dipakai sebagai sumber, bukan hasil generate, karena ia sudah standar lintas-agent; satu lapisan indireksi lebih sedikit.

### 6.2 Compiler (`tempa sync`)

Menghasilkan file target per agent. Setiap file hasil generate diawali header:
`<!-- generated by tempa from AGENTS.md and .tempa/ — do not edit; run "tempa sync" -->`
(atau komentar setara sesuai format file).

Matriks target **awal** — builder WAJIB memverifikasi lokasi dan format tiap baris terhadap dokumentasi resmi agent versi terkini sebelum implementasi, dan memperbarui matriks ini:

| Sumber | Claude Code | Codex | Gemini CLI | OpenCode | Cursor |
|---|---|---|---|---|---|
| AGENTS.md | `CLAUDE.md` berisi `@AGENTS.md` + bagian khusus Claude | native | `GEMINI.md` (atau setting context file → AGENTS.md) | native | `.cursor/rules/tempa.mdc` (alwaysApply) |
| commands | `.claude/commands/*.md` | lokasi prompt Codex (verifikasi) | `.gemini/commands/*.toml` | `.opencode/command/*.md` | `.cursor/commands/*.md` |
| agents | `.claude/agents/*.md` | gap → disajikan lewat `tempa_context` | gap/verifikasi | `.opencode/agent/*.md` | gap/verifikasi |
| skills | `.claude/skills/` | lokasi skills Codex (verifikasi) | verifikasi | verifikasi | verifikasi |

Target tambahan **[call]**:
- **MCP server proyek:** `.tempa/mcp.toml` di-compile ke config MCP proyek tiap agent (mis. `.mcp.json` untuk Claude, setting MCP proyek Gemini/OpenCode/Cursor/Codex — lokasi diverifikasi seperti matriks di atas). Di dalam aplikasi, server yang sama juga disuntikkan ke `session/new` bersama server `tempa`. Tujuannya: tool yang sama untuk semua agent, juga di luar aplikasi.
- **Setting agent:** compiler menghasilkan setting proyek tiap agent yang mendukungnya, minimal deny rule untuk `protected_paths` (mis. `.claude/settings.json` `permissions.deny`), digabung dengan isi `.tempa/targets/<agent>/settings.*` milik agent itu. Sehingga perlindungan path juga berlaku saat agent dijalankan di luar aplikasi.

Aturan untuk sel "gap": item tetap disajikan ke agent lewat tool MCP host `tempa_context` (bagian 7) dan dilaporkan oleh `tempa doctor`. Tidak boleh ada item yang hilang diam-diam. **[call]**

### 6.3 Importer (`tempa import`)

Brownfield pertama. Membaca, lalu menulis ke sumber kanonik:

| Masukan | Tujuan |
|---|---|
| `CLAUDE.md` (termasuk file yang di-`@import`) | digabung ke `AGENTS.md`; bagian yang jelas khusus Claude (sintaks/perilaku Claude) masuk blok khusus Claude di `tempa.toml` atau `.tempa/targets/claude.md` |
| `AGENTS.md`, `GEMINI.md`, `.cursor/rules/*`, `.cursorrules`, `.github/copilot-instructions.md` yang sudah ada | digabung; duplikasi dideteksi dan ditampilkan untuk diputuskan manusia |
| `.claude/commands/` | `.tempa/commands/` |
| `.claude/agents/` | `.tempa/agents/` |
| `.claude/skills/` | `.tempa/skills/` |
| `.claude/settings.json` `permissions.deny/allow` | `[gates].protected_paths` / allowlist |
| `.claude/settings.json` `hooks` | diklasifikasi: perintah verifikasi (lint/test/format) → **disalin** ke `[harness]`; aturan blokir → **disalin** ke `[gates]`; hook aslinya **tetap dipertahankan** di target Claude (`.tempa/targets/claude/settings.json`) agar perilaku engineer Claude seperti auto-format tidak hilang; sisanya → laporan gap **[call]** |
| `.mcp.json` dan config MCP proyek agent lain | `.tempa/mcp.toml` (satu daftar MCP server proyek) **[call]** |

Import bersifat interaktif di aplikasi (preview diff sebelum tulis) dan `--yes` di CLI. Tidak pernah menghapus file sumber lama sebelum file hasil generate menggantikannya. **[call]**

### 6.4 Anti-drift (A1)

- `lock.json` menyimpan hash file hasil generate.
- **Aplikasi:** watcher pada `.claude/`, `CLAUDE.md`, `GEMINI.md`, `.cursor/`, dan file target lain. Bila file hasil generate diedit langsung atau file baru muncul di folder target (mis. `.claude/commands/baru.md`), tampilkan banner "Serap ke sumber Tempa" → jalankan import untuk perubahan itu → sync.
- **CLI/CI:** `tempa check` gagal bila ada drift (hash beda atau file target yang tidak berasal dari sumber).
- **Git hook** dipasang `tempa init`: pre-commit `tempa sync --fix` **[user]**.
- **Siapa yang sync [user]:** arah sumber → target (AGENTS.md/.tempa → CLAUDE.md, .gemini/, dst.) otomatis oleh sistem (hook, dan watcher aplikasi di M2). Arah target → sumber (edit langsung ke file hasil generate) tidak pernah otomatis: sistem memberi alert, user menyerap lewat `tempa import`. Isi dokumen kontrak ditulis manusia/agent; sistem hanya memvalidasi dan memberi alert. CI hanya memeriksa, tidak pernah menulis.
- Memory pribadi agent di luar repo (mis. auto-memory user-level) di luar cakupan; pengetahuan proyek harus masuk lewat dokumen bersama (bagian 7).

## 7. Lapisan Sesi: Tempa Host MCP

Aplikasi menyuntikkan MCP server `tempa` ke setiap ACP `session/new` (mekanisme dari Termul `host_mcp`). Tool-nya sama untuk semua agent:

| Tool | Fungsi |
|---|---|
| `tempa_context` | Mengembalikan konteks hasil compile (AGENTS.md + commands/agents/skills yang relevan) — penutup gap untuk agent yang tidak membaca format tertentu |
| `tempa_plan` | Menulis/menimpa plan task (file per task, bagian 9) dan memperbarui PlanPanel |
| `tempa_decide` | Menulis decision record dengan template |
| `tempa_log_change` | Menambah entri changes dengan template |
| `tempa_check` | Meminta host menjalankan harness; hasil dicatat host, bukan dilaporkan agent |
| `tempa_handoff` | Meminta host membuat PR dari task saat semua gate lulus |

Tool yang menulis dokumen memvalidasi input terhadap kontrak (bagian 9) dan menolak dengan pesan perbaikan bila bagian wajib kosong. Skema JSON persis tiap tool diserahkan ke builder.

Prompt pertama setiap sesi diawali preamble pendek dari host: aturan lifecycle + daftar tool `tempa_*` + id task. **[call]**

## 8. Lifecycle task (G2)

**Sumber kebenaran state task adalah repo [call]:** frontmatter file plan (`task_id`, `state`, `approved_by`, `approved_at`, `implementer`, `reviewer`) di branch `tempa/<task-id>`. SQLite lokal hanya cache dan riwayat sesi. Akibatnya task bisa dilanjutkan engineer lain atau di mesin lain: checkout branch → aplikasi/CLI membangun ulang state dari file plan. Konflik dua orang di task yang sama diselesaikan seperti konflik git biasa.

**Kerja di luar task [call]:**
- Chat agent tanpa task diizinkan sebagai mode **Eksplorasi**: perlakuan sama dengan state sebelum `Approved` (edit/delete/move ditolak). Tombol "Jadikan task" memindahkan percakapan ke task baru.
- Agent yang dijalankan langsung di tab terminal (mis. mengetik `claude`) diizinkan. Aplikasi mendeteksi nama binary agent v1 yang dikenal di PTY lalu menampilkan banner "Sesi ini di luar lifecycle Tempa; CI tetap memeriksa kontrak". Tidak diblokir.

State machine:

```
Intake → Plan → Approved → Implement → Verify ⇄ Implement → Record → Review → Handoff → Done
```

| Transisi | Syarat |
|---|---|
| Intake → Plan | task dibuat (judul, deskripsi); worktree + branch baru `tempa/<task-id>` dibuat |
| Plan → Approved | plan lolos kontrak; **manusia** menekan Approve di aplikasi |
| Approved → Implement | otomatis |
| Implement → Verify | agent memanggil `tempa_check` atau engineer menekan Verify |
| Verify → Record | semua perintah harness lulus |
| Record → Review | entri changes ada; decision ada bila diff memicu `decision_required_for` |
| Review → Handoff | bila `gates.review_required`: review oleh agent/model berbeda dari implementer sudah tercatat |
| Handoff → Done | PR dibuat |

Penegakan oleh host (adaptasi `tool_policy.rs`):
- Sebelum `Approved`: permission untuk kind `edit`, `delete`, `move` ditolak; `read`, `search`, `fetch` diizinkan; `execute` diteruskan ke engineer seperti biasa. **[call]**
- `protected_paths` selalu ditolak untuk agent, di semua state.
- Host tidak pernah memilih opsi "allow always" dan tidak mengizinkan agent mengganti mode.
- Batasan diketahui: sebagian agent (Claude Code) bisa membaca/mengedit di cwd tanpa meminta izin pada mode tertentu. Tempa meluncurkan Claude dengan mode yang selalu meminta izin + deny rule untuk protected path (pola Termul `with_headless_options`). Penegakan final tetap di gate CI.

Deteksi `decision_required_for` v1 berbasis diff sederhana: `new-dependency` = perubahan file manifest dependensi; `schema-change` = path yang cocok glob di config; `public-api` = path glob di config. **[call]**

Review lintas-model: engineer memilih agent/model reviewer; host mengirim diff + plan + kontrak; hasil review disimpan ke bagian Review di body PR. **[call]**

## 9. Kontrak dokumen (A2)

Standar diikat ke **peran**, nama file bebas di `tempa.toml`. **[user: nama bisa disesuaikan]**

| Peran | Default | Bagian wajib | Penulis |
|---|---|---|---|
| context | `AGENTS.md` | Stack, Perintah, Konvensi, Area terlarang | manusia |
| plan | `docs/plans/<task-id>.md` | Tujuan, Langkah (dengan status), File yang disentuh, Risiko | `tempa_plan` |
| decisions | `docs/decisions/<YYYY-MM-DD>-<slug>.md` | Konteks, Opsi, Keputusan, Konsekuensi, Agent+model | `tempa_decide` |
| changes | `CHANGES.md` | per entri: tanggal, task-id, ringkasan, alasan, link decision, hasil harness, agent+model | `tempa_log_change` |
| harness | `HARNESS.md` | Definisi selesai, Gate yang berlaku | manusia |

Keputusan untuk kerja paralel tanpa konflik merge **[call]**:
- Plan dan decision satu file per task/keputusan, id berbasis tanggal+slug (bukan nomor urut yang bisa bentrok).
- `CHANGES.md` append-only dengan `.gitattributes`: `CHANGES.md merge=union` (dipasang `tempa init`).

Validasi struktural saja (heading wajib ada dan tidak kosong, link decision valid), bukan gaya tulisan.

### 9.1 `tempa.toml`

```toml
[project]
name = "billing-api"
# extends = "git+https://bitbucket.org/acme/tempa-policy@v1"   # v1: tidak didukung, lihat 13

[docs]
context   = "AGENTS.md"
plan      = "docs/plans/"
decisions = "docs/decisions/"
changes   = "CHANGES.md"
harness   = "HARNESS.md"

[agents]
enabled = ["claude", "codex", "gemini", "opencode", "cursor"]

[harness]            # dijalankan berurutan di worktree task; semua harus exit 0
format    = "bun run format:check"
lint      = "bun run lint"
typecheck = "bun run typecheck"
test      = "bun run test"

[gates]
plan_before_edit      = true
review_required       = false
decision_required_for = ["new-dependency", "schema-change", "public-api"]
schema_paths          = ["migrations/**"]
public_api_paths      = ["src/api/**"]
max_diff_lines        = 800
protected_paths       = [".env*", "bitbucket-pipelines.yml", "tempa.toml"]
```

### 9.2 Kontrak PR

Body PR dibuat host dari template tetap: Ringkasan, Link plan, Decision terkait, Entri changes, Hasil harness (per perintah: lulus/gagal + durasi), Review, Agent + model (implementer dan reviewer). Body PR divalidasi secara lokal (aplikasi dan `tempa pr-body`), **tidak** di CI pada v1, supaya step DevOps tidak butuh token API Bitbucket. **[call]**

## 10. CLI `tempa`

| Perintah | Fungsi |
|---|---|
| `tempa init` | buat `tempa.toml`, `HARNESS.md`, folder docs, `.gitattributes`, git hook opsional |
| `tempa import [--yes]` | bagian 6.3 |
| `tempa sync [--check]` | compile ke target; `--check` hanya cek drift |
| `tempa check` | drift + validasi kontrak dokumen + harness (+ body PR bila ada); exit non-zero bila gagal |
| `tempa doctor` | laporan paritas per agent: terbaca / ditegakkan host / gap / tingkat paritas |
| `tempa task new <slug>` | jalur di luar aplikasi: buat branch `tempa/<task-id>` + file plan dari template **[user: jalur luar app harus ada]** |
| `tempa task approve` | isi `approved_by`/`approved_at` di plan (panduan proses, lihat 3.1) |
| `tempa decide <slug>` | buat decision record dari template |
| `tempa log` | tambah entri changes dari template, termasuk agent+model (diisi manual atau dari flag) |
| `tempa pr-body` | cetak body PR dari template + data task, untuk ditempel ke Bitbucket |
| `tempa check --contract-only` | drift + kontrak dokumen saja, tanpa harness (dipakai di Pipelines) |

Cakupan `tempa check` di CI **[user + call]**: berjalan di **semua PR** dengan kontrak yang sama. Pengecualian lewat config:

```toml
[ci]
skip_authors = ["dependabot[bot]", "renovate[bot]"]   # tetap jalankan harness, lewati kontrak dokumen
docs_only_paths = ["docs/**", "*.md"]                 # PR yang hanya menyentuh path ini: lewati plan/changes
```

Di luar aplikasi, struktur body PR yang sama didapat dari `tempa pr-body`. Harness di CI dijalankan oleh step DevOps yang sudah ada; bagian "Hasil harness" di body PR bersifat informasi.

Snippet untuk DevOps (dihasilkan `tempa init` di `docs/tempa-devops.md`) **[call]**:

```yaml
# tambahkan ke bitbucket-pipelines.yml yang sudah ada
pipelines:
  pull-requests:
    '**':
      - step:
          name: tempa contract
          script:
            - curl -fsSL <url-installer-tempa> | sh   # atau image Docker tempa yang di-pin versinya
            - tempa check --contract-only
```

Distribusi binary untuk CI: image Docker kecil dengan versi ter-pin, selain installer script. Versi `tempa` yang dipakai CI dikunci di `tempa.toml` (`[project].tempa_version`) agar lokal dan CI tidak berbeda. **[call]**

## 11. Aplikasi desktop v1

Layar/area:
- Sidebar proyek (folder repo) dan daftar task per proyek dengan badge state lifecycle.
- Terminal: tab dan split pane (dari Termul).
- Chat agent ACP per task: pilih agent + model, permission prompt, PlanPanel.
- **Panel Kontrak:** status tiap peran dokumen, drift + tombol serap, ringkasan `tempa doctor`.
- **Panel Task:** state lifecycle, preview plan markdown + tombol Approve, hasil harness, tombol Review (pilih reviewer), tombol Handoff.
- Scorecard lokal per agent/model: lulus gate di percobaan pertama, jumlah putaran Verify gagal, ukuran diff, durasi. Disimpan lokal saja. **[call]**

Detail desain UI diserahkan ke builder.

## 12. Kualitas produksi (G3)

- Build ter-sign dan ter-notarize: macOS (Developer ID + notarization), Windows (code signing), Linux (AppImage + deb).
- Auto-update via Tauri updater dengan artefak ter-sign.
- Semua tulis file milik Tempa atomik; state lifecycle di SQLite dengan migrasi berversi; crash tidak boleh merusak repo atau state.
- Test: unit test seluruh `tempa-core`; golden test untuk importer dan compiler (fixture repo → output yang diharapkan per agent); test state machine lifecycle dan tool policy; e2e nightly menjalankan agent sungguhan pada repo fixture untuk tiap agent v1.
- Tanpa telemetri jarak jauh secara default.
- Log terstruktur lokal dengan rotasi.

## 13. Urutan milestone [call]

| M | Isi | Bisa dipakai sendiri? |
|---|---|---|
| M1 | `tempa-core` + CLI: init, import, sync, check (+ `--contract-only`), doctor, task/decide/log/pr-body + hook L2 + `docs/tempa-devops.md` + image Docker CI; matriks target terverifikasi | Ya — sudah menutup G1, jalur luar-app, dan gate CI |
| M2 | Shell desktop: proyek, PTY, terminal tab/pane, panel Kontrak + watcher anti-drift | Ya |
| M3 | ACP client + Tempa Host MCP + lifecycle + tool policy + worktree per task | Ya — menutup G2 |
| M4 | Review lintas-model, handoff PR, scorecard, signing, auto-update, e2e nightly | Rilis 1.0 |

Ditunda setelah 1.0: policy pack organisasi (`extends`), dashboard tim, agent di luar 5 agent v1 dengan jaminan paritas.

## 14. Diserahkan ke builder

- Struktur internal crate dan modul, nama tipe.
- Skema JSON persis tool `tempa_*`.
- Desain visual dan detail interaksi UI.
- Pilihan library test, logging, dan SQLite selama sesuai stack.
- Verifikasi dan pembaruan matriks target (6.2) — wajib dilakukan di M1 sebelum menulis compiler.
- Format teks preamble sesi.
