# Contributing

Thanks for helping. tempa is developed with tempa, so a change follows the same workflow it enforces.

## Setup

- Rust 1.85+ and git. The pre-push hook also runs the tests of the frozen desktop app in `app/`, which need [Bun](https://bun.sh) and the [Tauri system dependencies](https://tauri.app/start/prerequisites/).
- `cargo install --path crates/tempa-cli --locked`, then `tempa init` once per clone to install the hooks.

## A change

1. Open an issue first for anything bigger than a bug fix, so we agree on the direction.
2. `tempa task new <slug>`, fill in the plan (`docs/plans/<id>.md`) and approve it with `tempa task approve`.
3. Write a failing test first, then the code. See `AGENTS.md` for conventions and `HARNESS.md` for the definition of done.
4. Changing a generated format for an agent? Update `docs/target-matrix.md` and link the agent's official docs.
5. `tempa log ...`, commit (the pre-commit hook syncs generated files), push (the pre-push hook runs `tempa check`).
6. Open the pull request with `tempa pr-body` as its description. Keep it under 800 changed lines; split larger work.

## License

By contributing you agree that your contribution is dual licensed under MIT OR Apache-2.0, as described in the README.
