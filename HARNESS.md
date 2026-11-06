# Harness

## Definisi selesai

- `tempa check` lulus: kontrak dokumen, aturan task, dan seluruh harness di `tempa.toml`.
- Test baru untuk setiap perilaku baru atau bug yang diperbaiki, dan test itu terbukti gagal sebelum perbaikan.
- `scripts/e2e.sh` lulus untuk perubahan yang menyentuh alur CLI.
- Entri `CHANGES.md` dibuat lewat `tempa log`, mencantumkan agent dan model.

## Gate yang berlaku

- Pre-commit: `tempa sync --fix` (sync dari sumber yang sudah di-stage, blokir drift).
- Pre-push: `tempa check` (kontrak + harness: format, lint, test CLI; typecheck, Vitest, clippy dan test backend aplikasi `app/`). Butuh Bun dan dependency sistem Tauri.
- CI: `.github/workflows/app-windows.yml` menjalankan test backend aplikasi di Windows untuk PR yang menyentuh `app/` atau `tempa-core`. Selebihnya reviewer menjalankan `tempa check --base main` di branch PR sebelum approve.
- Perubahan UI aplikasi: dicek manual di aplikasi yang berjalan (`cd app && bun tauri dev`) sebelum review akhir milestone.
