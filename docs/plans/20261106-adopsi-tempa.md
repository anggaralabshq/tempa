---
task_id: 20261106-adopsi-tempa
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

Memakai tempa di repo tempa sendiri (dogfooding) sebagai pilot M1: kontrak nyata, hook aktif, dan bukti agent asli membaca kontrak.

## Langkah

1. `tempa init`, isi AGENTS.md, HARNESS.md, dan harness cargo di tempa.toml.
2. Command `.tempa/commands/review.md`, lalu `tempa sync`.
3. Uji Claude Code (`claude -p`) dan OpenCode (`opencode debug config`) membaca kontrak.
4. Catat temuan pilot ke backlog.

## File yang disentuh

tempa.toml, AGENTS.md, HARNESS.md, CHANGES.md, .tempa/, file target hasil generate, .gitattributes, docs/tempa-devops.md, docs/plans/.

## Risiko

Kode produk tidak berubah. Hook pre-push menjalankan seluruh test, sehingga push jadi lebih lambat.
