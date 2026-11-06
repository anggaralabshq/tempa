# tempa

Satu kontrak konteks untuk semua coding agent. Tulis konteks proyek sekali di `AGENTS.md` dan `.tempa/`, lalu `tempa` meng-generate file yang dibaca Claude Code, Codex, Gemini CLI, OpenCode, dan Cursor. Engineer mana pun, dengan agent apa pun, mendapat konteks yang sama.

## Instalasi

```sh
cargo install --path crates/tempa-cli --locked
```

Butuh Rust 1.85+ dan git. Installer biner menunggu keputusan hosting.

## Alur

```sh
tempa import --yes      # serap CLAUDE.md, .claude/, .cursorrules, dll. ke sumber netral (+ init bila belum ada tempa.toml)
tempa init              # repo baru: tempa.toml, AGENTS.md, HARNESS.md, CHANGES.md, git hooks
tempa task new <slug>   # branch tempa/<id> + file plan; isi plan lalu `tempa task approve`
tempa log --summary .. --reason .. --agent "codex / gpt-5"   # entri CHANGES.md
tempa pr-body           # body PR untuk Bitbucket
```

Pemeriksaan:

- `tempa sync --check`: target agent sama dengan sumber? Hook pre-commit menjalankan `tempa sync --fix` otomatis.
- `tempa check`: drift, kontrak dokumen, aturan task, dan harness. Hook pre-push menjalankannya. CI memakai `--contract-only`.
- `tempa doctor`: paritas per agent, gap, dan file legacy.

Jangan edit file hasil generate (`CLAUDE.md`, `.claude/commands/`, `.gemini/`, dll.) secara langsung. Ubah sumbernya, atau serap editan dengan `tempa import --yes`. `tempa sync` tidak pernah menimpa file yang bukan buatannya atau yang sudah diedit; `tempa sync --force` menimpanya dengan sengaja.

Hook pre-commit hanya men-sync dari sumber yang sudah di-stage (`git add`), dan ikut men-stage `.tempa/lock.json`.

## CI dan merge

Serahkan [docs/tempa-devops.md](docs/tempa-devops.md) (dibuat `tempa init` di repo proyek) ke tim DevOps: satu step Bitbucket Pipelines dan merge check Premium.

Image: `docker build -t tempa:0.1.0 .`

Matriks target per agent: [docs/target-matrix.md](docs/target-matrix.md).

## Lisensi

Belum diputuskan.
