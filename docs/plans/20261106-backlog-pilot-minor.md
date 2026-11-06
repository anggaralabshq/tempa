---
task_id: 20261106-backlog-pilot-minor
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Menutup temuan pilot (P1–P6) dan minor yang ditunda dari review akhir M1 (D1–D7).

## Langkah

- P1–P3 pr-body: hanya decision .md, hasil harness dari entri changes (bukan teks Bitbucket), heading entri diturunkan ke ###.
- P4 output harness ditangkap; saat gagal tampil 20 baris terakhir.
- P5 default tempa.toml netral platform/stack (contoh CI sebagai komentar).
- P6 warning harness.empty.
- D1 skip_authors pakai --no-merges. D2 harness Windows lewat raw_arg. D3 pesan error jelas (task new di luar git, config tak terbaca, nama proyek di-escape).
- D4 sintaks command Claude → Gemini (!{} / @{}), $1..$9 dilaporkan gap. D5 hook pengubah file bukan harness. D6 fence ``` vs ~~~ dan heading berindentasi. D7 e2e langkah 8: dua clone.

## File yang disentuh

crates/tempa-core/src/{task,check,config,gitx,import,md,targets}.rs, crates/tempa-core/tests/{backlog_a,check,task,config,targets}.rs, scripts/e2e.sh.

## Risiko

Default config berubah: repo baru tidak lagi melindungi bitbucket-pipelines.yml otomatis (contoh tetap ada di komentar). D2 hanya diverifikasi compile untuk target Windows, belum dijalankan di Windows.
