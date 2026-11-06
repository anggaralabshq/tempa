# Tempa M2 (desktop shell) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A Tauri desktop app where each project gets terminals (tabs, split panes) and a Contract panel, with a watcher that syncs source changes and alerts on target edits.

**Architecture:** `app/src-tauri` is a separate Cargo workspace (crate `tempa-app`) depending on `tempa-core` by path; Tauri commands call `tempa-core` directly and return app DTOs. `app/src` is React + TypeScript (Vite, Bun, Tailwind, zustand, xterm.js). Each task below is one PR (≤ 800 lines, lockfiles excluded after Task 1) through tempa's own task cycle.

**Tech Stack:** Rust (Tauri 2, portable-pty, notify), React 18 + TypeScript, Vite, Bun, Tailwind 4, zustand, @xterm/xterm 5 (stable) + fit/web-links addons, Vitest.

**Spec:** `docs/superpowers/specs/2026-11-06-tempa-m2-desktop-design.md` (approved), main spec sections 5, 6.4, 11.

## Global Constraints

- `crates/` keeps MSRV 1.85 and builds without any app toolchain; root workspace `exclude = ["app/src-tauri"]`.
- No new dependency in `tempa-core`/`tempa-cli`. App dependencies pinned to exact versions in `package.json`; Cargo versions as in Termul (`tauri 2.x`, `portable-pty 0.9`, `notify 8`).
- All user-facing text in English. Contract vocabulary unchanged.
- The app never writes target → source on its own; absorb/discard/init/import/install-hooks run only on a click, after a confirmation that lists affected files.
- Every contract action calls the same `tempa-core` function as the CLI command of the same name.
- Copied Termul code keeps its MIT notice in `THIRD_PARTY_NOTICES.md`, with the source path.
- OS: macOS and Windows must work (Windows backend tests on GitHub Actions `windows-latest`); Linux compiles.
- Before each commit on a branch, reinstall the CLI from that branch (`cargo install --path crates/tempa-cli --locked --force`) so the pre-commit hook matches the source.

## Review Focus

1. **PTY lifecycle:** closing a pane, a tab, a project or the app kills its shell (no orphan processes); a shell that exits on its own closes cleanly; output after kill is not delivered to a dead xterm.
2. **Watcher loops and noise:** tempa's own sync writes must not trigger another sync or an alert; editor save patterns (write temp + rename, many events per save) produce one sync; `.git/` and ignored files are ignored.
3. **PATH on a GUI launch:** app started from Finder/Dock (no shell env) still finds `tempa`, `claude`, `codex` in terminals.
4. **Folder states:** folder without `tempa.toml` (terminal-only, offers Init or Import), folder that is not a git repo, folder deleted or moved while the app runs.
5. **Destructive actions:** Discard (`sync_force`) and Absorb (`import --yes`) list every affected file before running and are never triggered by the watcher.

---

### Task 1: Lockfiles do not count toward the diff gate

**Files:** Modify `crates/tempa-core/src/check.rs` (exclude set before `gitx::diff_lines`); test `crates/tempa-core/tests/check.rs`.

**Produces:** `pub const LOCKFILES: [&str; 11] = ["Cargo.lock", "bun.lock", "bun.lockb", "package-lock.json", "yarn.lock", "pnpm-lock.yaml", "poetry.lock", "uv.lock", "Gemfile.lock", "composer.lock", "go.sum"]` in `check.rs`; any changed path whose file name is in the list is excluded (any directory).

- [ ] Test `diff_gate_ignores_lockfiles`: valid repo, task branch, commit 900 lines to `app/bun.lock` and 10 lines to `src/x.rs` → no `task.diff_too_large`; 900 lines to `src/big.rs` → the finding is present. Run, watch it fail.
- [ ] Implement: `gitx::diff_lines` receives a predicate or the check builds the exclude set from `git diff --name-only`; keep `diff_lines` signature if possible.
- [ ] `cargo test --workspace` green; README "Rules worth knowing" line: lockfiles are not counted.
- [ ] Task cycle, PR, merge.

### Task 2: App scaffold

**Files:** Create `app/` (Tauri 2 + React + TS via Vite, Bun), `app/src-tauri/Cargo.toml` (own `[workspace]`, `tempa-core = { path = "../../crates/tempa-core" }`), `THIRD_PARTY_NOTICES.md`; modify root `Cargo.toml` (`exclude`), `.gitignore` (`app/node_modules`, `app/dist`, `app/src-tauri/target`).

**Produces:** `bun run check` (biome or `tsc --noEmit`), `bun run test` (vitest), `bun tauri dev` opens a window titled "tempa" with a three-column shell (sidebar, main, contract) and placeholder text. Command `app_version() -> String`.

- [ ] Vitest smoke test `App renders three regions` (RED before App exists).
- [ ] Scaffold; window opens on macOS (`bun tauri dev`, screenshot).
- [ ] Ask the owner to add to `tempa.toml` `[harness]`: `app_check = "cd app && bun install --frozen-lockfile && bun run check"`, `app_test = "cd app && bun run test"`, `app_rust = "cargo test --manifest-path app/src-tauri/Cargo.toml"`.
- [ ] Task cycle, PR, merge.

### Task 3: PTY backend, one terminal, PATH

**Files:** Create `app/src-tauri/src/{pty.rs,env.rs}` (env.rs adapted from Termul `src-tauri/src/pty/env_refresh.rs` + the Windows parts of `shell_paths.rs`), `app/src/terminal/Terminal.tsx`, `app/src/lib/ipc.ts`; propose `.github/workflows/app-windows.yml` (protected path: written only after the owner approves).

**Produces:**
- `env::login_path() -> Option<String>` (Unix: probe `$SHELL -l -c` with a timeout; Windows: registry PATH merged), `env::merge_path_segments(a: &str, b: &str, sep: char) -> String`.
- `PtyManager` (Tauri state): `spawn(cwd: &Path, cols: u16, rows: u16, on_data: Channel<Vec<u8>>, on_exit: Channel<i32>) -> Result<u32>`, `write(id, &[u8])`, `resize(id, cols, rows)`, `kill(id)`, `kill_all()` (called on app exit).
- Commands `pty_spawn { cwd, cols, rows, onData, onExit } -> u32`, `pty_write { id, data: string }`, `pty_resize { id, cols, rows }`, `pty_kill { id }`.
- Shell: `$SHELL` (fallback `/bin/zsh` macOS, `/bin/bash`) with `-l`; Windows: `pwsh`, then `powershell`, then Git Bash. `TERM=xterm-256color`, PATH from `login_path()`.

- [ ] Rust tests (RED first): `merge_path_segments_dedupes_and_keeps_order`; `spawn_echo_reports_output_and_exit` (spawn `sh -c 'echo hi; exit 3'` via an injectable command, collect bytes contain `hi`, exit 3; Windows: `cmd /C echo hi & exit 3`); `kill_ends_process`.
- [ ] Implement; terminal in the main region of the scaffold runs the login shell at the app's cwd.
- [ ] Manual: launch the built app from Finder; `which tempa claude` resolve in the terminal.
- [ ] Windows workflow: `cargo test --manifest-path app/src-tauri/Cargo.toml` on `windows-latest` (owner approves the file).
- [ ] Task cycle, PR, merge.

### Task 4: Tabs and split panes

**Files:** Create `app/src/layout/panes.ts` (pure), `app/src/layout/PaneView.tsx`, `app/src/layout/Tabs.tsx`; tests `app/src/layout/panes.test.ts`.

**Produces:** `type Pane = { kind: "leaf"; id: string } | { kind: "split"; dir: "row" | "col"; a: Pane; b: Pane }`; `split(tree, leafId, dir, newId): Pane`, `close(tree, leafId): Pane | null` (sibling takes the parent's place; `null` when the last leaf closes), `leaves(tree): string[]`. A tab = `{ id, title, root: Pane, focus: string }`. Shortcuts: new tab Cmd/Ctrl+T, split right Cmd/Ctrl+D, split down Cmd/Ctrl+Shift+D, close pane Cmd/Ctrl+W.

- [ ] Vitest (RED first): `split replaces the leaf with a split`, `close promotes the sibling`, `close of the last leaf returns null`, `leaves lists in visual order`.
- [ ] Each leaf owns one PTY; `close` and tab close call `pty_kill` for every removed leaf (Review Focus 1); a shell exiting on its own closes its pane.
- [ ] Task cycle, PR, merge.

### Task 5: Projects

**Files:** Create `app/src-tauri/src/projects.rs`, `app/src/store/projects.ts`, `app/src/sidebar/Sidebar.tsx`.

**Produces:** `Project { id: String, name: String, path: String }`; file `projects.json` in the app config dir, written atomically (`tempa_core::write_atomic`). Commands `projects_list() -> Vec<Project>`, `projects_add { path } -> Project` (folder picker via `tauri-plugin-dialog`; duplicate path returns the existing one; a missing folder is an error), `projects_remove { id }`. Each project keeps its own tabs; switching projects hides, not kills, its terminals; removing a project kills them.

- [ ] Rust tests (RED first): `add_is_idempotent_per_path`, `remove_persists`, `corrupt_file_starts_empty_and_is_kept_as_backup`.
- [ ] Vitest: `switching project keeps tabs of the other project`.
- [ ] Task cycle, PR, merge.

### Task 6: Contract panel

**Files:** Create `app/src-tauri/src/contract.rs`, `app/src/contract/ContractPanel.tsx`, `app/src/contract/ConfirmAction.tsx`; modify `crates/tempa-core/src/init.rs` (extract hooks).

**Produces:**
- `tempa_core::init::install_hooks(root: &Path) -> Result<bool>` (writes `.tempa/hooks/{pre-commit,pre-push}` if missing, sets `core.hooksPath` only when unset; returns whether it was set); `init` calls it. CLI behavior unchanged (existing init tests stay green).
- `contract_status { root } -> ContractStatus { has_config: bool, offer: Option<"init" | "import">, findings: Vec<FindingDto>, drift: Vec<FindingDto>, agents: Vec<AgentDto { agent, files, gaps, lifecycle }>, hooks_ok: bool, cli_version: Option<String>, app_version: String }` — findings from `check(contract_only: true)`, drift from `sync::drift_findings`, agents from `doctor`, `cli_version` from `tempa --version` on the login PATH.
- `contract_preview { root, action: "absorb" | "discard" } -> Vec<String>` (absorb: `import(root, false)` written + synced paths; discard: drift paths that `sync_force` would rewrite).
- `contract_run { root, action: "absorb" | "discard" | "init" | "import" | "hooks" } -> Vec<String>` (changed paths).
- UI per the design sketch; version warning when `cli_version != app_version`.

- [ ] Rust tests (RED first): `status_of_folder_without_config_offers_import_when_claude_md_exists`, `status_reports_drift_after_target_edit`, `preview_discard_lists_modified_targets`, `install_hooks_does_not_touch_contract_files`.
- [ ] Task cycle (public API change in tempa-core: decision record), PR, merge.

### Task 7: Watcher

**Files:** Create `app/src-tauri/src/watcher.rs`; modify `contract.rs`, `Sidebar.tsx`, `ContractPanel.tsx`.

**Produces:** `classify(rel: &str, cfg: &Config) -> Change { Source, Target, Ignore }` (Source: `AGENTS.md`/`[docs].context`, `.tempa/**` except `.tempa/lock.json` and `.tempa/hooks/**`, `tempa.toml`; Target: any path `targets::render_all` produces or a managed dir; Ignore: `.git/**`, `target/**`, `node_modules/**`, git-ignored); one `notify` watcher per project with config, debounce 300 ms; Source → `sync_with(root, all)` then emit `contract://changed { root, synced, conflicts }`; Target → emit `contract://changed` only when `drift` is non-empty; sidebar badge = drift count + error findings.

- [ ] Rust tests (RED first): `classify_table` (each kind incl. `.tempa/lock.json` → Ignore), `debounce_collapses_burst` (10 events in 50 ms → one flush), `own_sync_write_does_not_trigger_alert` (sync writes CLAUDE.md, drift empty → no event), `source_edit_syncs_target` (temp repo: edit AGENTS.md → CLAUDE.md content unchanged hash path synced, lock updated).
- [ ] Manual: edit `.tempa/commands/review.md` in an editor → `.gemini/commands/review.toml` updates without a click; edit `CLAUDE.md` directly → badge + Absorb offered.
- [ ] Task cycle, PR, merge.

### Final

- [ ] Fresh reviewer (most capable model) on the whole M2 range with this Review Focus; fix Critical/Important with RED→GREEN.
- [ ] Windows manual UI check by a teammate before closing M2 (owner arranges).
