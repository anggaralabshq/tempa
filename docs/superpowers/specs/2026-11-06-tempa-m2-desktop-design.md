# Tempa M2: shell desktop — desain

Status: draf, menunggu persetujuan. Builder check selesai; jawaban masuk sebagai [user]/[call]. Melengkapi spec utama (`2026-11-06-tempa-design.md`, bagian 5, 6.4, 11, 13).

Penanda: **[user]** = keputusan pemilik proyek. **[call]** = keputusan penulis, boleh ditolak.

## Intent

M1 dan M3-lite sudah menutup jalur CLI: konteks sampai ke kelima agent, dan dokumen workflow tervalidasi lewat `tempa mcp`. Yang belum ada adalah tempat engineer bekerja sehari-hari yang **menjaga kontrak tetap sinkron tanpa harus ingat menjalankan perintah**. M2 adalah shell desktop: terminal per proyek (tempat agent CLI apa pun dijalankan) ditambah panel Kontrak dan watcher yang menutup A1 (drift) secara langsung, bukan baru saat commit.

Tujuan M2 menurut spec utama: "Shell desktop: proyek, PTY, terminal tab/pane, panel Kontrak + watcher anti-drift". Bisa dipakai sendiri.

## Tujuan

1. **Proyek.** Tambah folder repo, daftar proyek tersimpan, pindah proyek tanpa kehilangan terminal yang sedang jalan.
2. **Terminal.** Shell login pengguna di root proyek. Tab dan split pane (kanan/bawah). Agent CLI (claude, codex, gemini, opencode, cursor-agent) dijalankan di sini seperti di terminal biasa, dan menemukan `tempa mcp` lewat config yang sudah di-generate.
3. **Panel Kontrak.** Untuk proyek aktif: status dokumen kontrak (AGENTS.md, HARNESS.md, CHANGES.md, folder plan/decision) dari temuan `tempa check --contract-only`; daftar drift dengan aksi; ringkasan `tempa doctor` per agent; status hook.
4. **Watcher anti-drift (spec 6.4).**
   - Sumber berubah (`AGENTS.md`, `.tempa/**`, `tempa.toml`) → `sync` otomatis (no-clobber yang sama dengan CLI). Konflik tampil di panel, tidak pernah ditimpa.
   - File target diedit langsung atau file asing muncul → badge + notifikasi, tombol **Serap** (`import --yes`) atau **Buang** (`sync --force`) di panel. Tidak pernah otomatis.
5. **Hook L2.** Saat proyek dibuka dan `core.hooksPath` belum diset, panel menawarkan pasang hook (fungsi `init` yang sama, tanpa menimpa file).

## Non-goal M2 (tetap di M3/M4)

- Chat agent ACP, host MCP yang disuntik, permission prompt, PlanPanel, tool policy, worktree per task (M3).
- Panel Task, tombol Approve, Review, Handoff (M3/M4).
- State lifecycle di SQLite (M3; M2 tidak punya state lifecycle sendiri, semua dibaca dari repo).
- Scorecard, signing, notarization, auto-update, e2e nightly (M4).
- Sesi terminal yang bertahan setelah aplikasi ditutup, remote/web, SSH, editor, browser (non-goal v1).

## Anti-goal

- Aplikasi dan CLI berbeda perilaku. Semua aksi kontrak memanggil `tempa-core` yang sama; aplikasi tidak menambah aturan sendiri.
- Watcher menulis ke file milik pengguna. Arah target → sumber tetap manual.
- Terminal yang kehilangan PATH pengguna (masalah klasik aplikasi GUI macOS): agent dan `tempa` tidak ditemukan.

## Arsitektur [call]

```
tempa/
├─ crates/tempa-core, crates/tempa-cli   # tidak berubah perannya
└─ app/
   ├─ src-tauri/   # crate `tempa-app`: PTY, watcher, command Tauri → tempa-core
   ├─ src/         # React + TypeScript (Vite)
   └─ package.json # Bun
```

- Stack sesuai spec: Tauri 2, React + TypeScript, Bun, Vite, Tailwind, zustand, xterm.js (versi stabil, bukan beta seperti Termul).
- `app/src-tauri` adalah workspace Cargo **terpisah** (di-`exclude` dari workspace root) yang bergantung pada `tempa-core` lewat path. Alasan: `cargo test --workspace`, image CI `rust:1.85-alpine`, dan MSRV CLI tidak ikut menarik dependency Tauri/WebKit. CLI tetap bisa dibangun tanpa toolchain aplikasi.
- Satu proses backend per aplikasi; tiap proyek punya watcher sendiri; tiap pane punya satu PTY.

### Yang dipinjam dari Termul [user: pinjam; call: daftar]

Survei Termul v0.4.52: modul PTY-nya (`pty/manager/*`, ~5.700 baris) terjalin dengan klaim kredensial, ledger, relay web, dan sesi yang dipertahankan — fitur yang non-goal untuk tempa. Menyalinnya utuh membawa kompleksitas itu. Jadi:

| Kebutuhan | Dari Termul | Cara |
|---|---|---|
| PATH login shell untuk aplikasi GUI | `pty/env_refresh.rs` (`probe_unix_login_path`, `merge_path_segments`, registry PATH Windows) | salin + adaptasi, catat di `THIRD_PARTY_NOTICES.md` |
| Shell default Windows (Git Bash / PowerShell) | `shell_paths.rs` | salin bagian yang dipakai |
| PTY | pola `portable-pty` + Tauri `Channel` untuk byte mentah | tulis ulang kecil (spawn, write, resize, kill, exit) |
| Terminal UI | konfigurasi xterm.js + addon fit/web-links | pola, bukan salinan komponen |

## Tampilan [call]

```
┌──────────────┬─────────────────────────────────────┬────────────────────────┐
│ PROJECTS     │ [zsh] [claude] [+]          ⊟ ⊞     │ CONTRACT          tempa│
│ ● tempa   2⚠ │ ┌────────────────┬────────────────┐ │ Documents              │
│   api        │ │ $ claude       │ $ tempa check  │ │  ✓ AGENTS.md           │
│   web        │ │ ...            │ ...            │ │  ✗ HARNESS.md  empty:  │
│              │ │                │                │ │    Gate yang berlaku   │
│              │ └────────────────┴────────────────┘ │ Drift (2)              │
│              │                                     │  CLAUDE.md  edited     │
│              │                                     │   [Absorb] [Discard]   │
│ + Add project│                                     │ Agents   files gaps mcp│
│              │                                     │  claude    4    0   ✓  │
│              │                                     │ Hooks ✓   Synced 12:04 │
└──────────────┴─────────────────────────────────────┴────────────────────────┘
```

- Badge di sidebar = jumlah drift + error kontrak, supaya proyek yang tidak aktif pun kelihatan bermasalah.
- Panel Kontrak bisa disembunyikan; terminal tetap area utama.
- Tema gelap mengikuti sistem; tidak ada kustomisasi tema di M2.

## Perilaku yang ditentukan

- **Spawn shell:** `$SHELL -l` (Unix) / PowerShell, fallback Git Bash (Windows); cwd = root proyek; env = env aplikasi + PATH login shell; `TERM=xterm-256color`. Pane ditutup → proses dibunuh. Aplikasi ditutup → semua PTY dibunuh.
- **Watcher:** `notify` dengan debounce 300 ms; mengabaikan `.git/`, `target/`, `node_modules/`, dan path yang di-ignore git. Penulisan oleh tempa sendiri (sync) tidak memicu alert (dibandingkan dengan lock hash, sama seperti `drift`).
- **Sync otomatis** hanya untuk perubahan sumber dan hanya bila hasilnya tanpa konflik; konflik menghasilkan temuan di panel, bukan dialog.
- **Aksi panel** memanggil fungsi `tempa-core` langsung (bukan menjalankan binary `tempa`), sehingga versi aturan = versi aplikasi. Hook pre-commit tetap memakai binary di PATH; panel memperingatkan bila `tempa --version` di PATH berbeda dari versi aplikasi (temuan pilot M3-lite).
- **Daftar proyek** disimpan sebagai JSON di direktori config aplikasi, ditulis atomik.
- **Serap/Buang berlaku untuk seluruh proyek, bukan per file [call].** Persis `tempa import --yes` dan `tempa sync --force`, jadi aplikasi dan CLI sama. Sebelum jalan, dialog konfirmasi mendaftar semua file yang terdampak. File yang tidak bisa diserap (`absorbs_edit` false) atau file asing di folder milik tempa ditampilkan dengan instruksi perbaikan dari temuan `tempa-core`, tanpa tombol. Aksi per file menyusul bila terasa perlu.
- **Folder tanpa `tempa.toml` boleh ditambahkan [call]** sebagai proyek terminal saja. Panel Kontrak menawarkan **Import** bila ada konteks agent (`init::existing_agent_context`), selain itu **Init**. Watcher baru aktif setelah `tempa.toml` ada.
- **Pasang hook = hook saja [call].** Fungsi baru kecil di `tempa-core` (dipakai juga oleh `tempa init`) yang hanya menulis `.tempa/hooks/` dan men-set `core.hooksPath` bila belum diset; tidak membuat dokumen kontrak.
- **Sync otomatis = perilaku `tempa sync` [call]:** tulis file yang aman, laporkan konflik di panel. Tidak ada mode "semua atau tidak sama sekali".

## Kualitas dan test [call]

- Backend: unit test untuk klasifikasi event watcher (sumber/target/abaikan), debounce, PATH merge, dan PTY (spawn `echo`, baca output, exit code) di macOS/Linux.
- Frontend: Vitest untuk reducer tata letak pane (split, tutup, fokus) dan store proyek.
- Verifikasi manual tiap PR: jalankan aplikasi (`bun tauri dev`), screenshot alur utama.
- **OS M2: macOS dan Windows [user].** Linux: kode tetap ditulis dan di-compile-check, dijamin di M4.
  - macOS: uji otomatis + uji manual di mesin ini.
  - Windows: test backend (PTY ConPTY, PATH registry, shell default) jalan di runner GitHub Actions `windows-latest` [call]. File workflow ada di `.github/workflows/` (protected path), jadi pemilik proyek yang menyetujui/menambahkannya. Uji manual UI di Windows oleh anggota tim yang memakai Windows sebelum review akhir M2.
- **Perintah aplikasi masuk `[harness]` [user]** (`bun run check`, `bun run test`, `cargo test --manifest-path app/src-tauri/Cargo.toml`), wajib untuk semua. `tempa.toml` protected, jadi pemilik proyek menambahkannya saat PR scaffold siap.

## Gate diff dan urutan PR [call]

Gate repo `max_diff_lines = 800` menghitung lockfile (`bun.lock`, `Cargo.lock` aplikasi: ribuan baris). Itu bukan kode yang direview. PR pertama M2 mengubah `tempa check` agar lockfile umum tidak dihitung (perilaku produk, berguna untuk semua pengguna tempa). Urutan, masing-masing ≤ 800 baris:

1. `tempa check`: lockfile tidak dihitung di diff.
2. Scaffold `app/` (Tauri + React + Bun), workspace terpisah, jendela kosong, `THIRD_PARTY_NOTICES.md`.
3. PTY backend + satu terminal + PATH login shell.
4. Tab dan split pane.
5. Proyek: sidebar, tambah/hapus, persistensi, terminal per proyek.
6. Panel Kontrak: dokumen, drift + Serap/Buang, doctor, hook, peringatan versi.
7. Watcher: sync otomatis sumber, alert target, badge sidebar.

Review akhir serius (reviewer baru) setelah PR 7, seperti milestone sebelumnya.

## Diserahkan ke builder

Nama tipe/command Tauri, struktur komponen React, warna dan ikon, library kecil UI selama tanpa dependency berat.
