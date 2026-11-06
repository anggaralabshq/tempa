# Tempa M3 (agents in the lifecycle) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The desktop app runs Claude Code, Codex, Gemini CLI, OpenCode and Cursor over ACP, one session per task in its own worktree, with an injected `tempa` MCP server, a host that answers permission requests by lifecycle state, and harness results recorded by the host.

**Architecture:** New backend modules in `app/src-tauri/src/` (`acp/`, `tasks.rs`, `verify.rs`, `mcp_host.rs`); the app binary doubles as the injected MCP server (`tempa-app --mcp`). ACP is a hand-written JSON-RPC client over `serde_json` (same approach as the M3-lite MCP server). Task state stays in the plan frontmatter in the repo; the host keeps only `verify.json` and session transcripts (JSONL) in the app data dir. Each task below is one PR (≤ 800 lines) through tempa's task cycle.

**Tech Stack:** Rust (Tauri 2, serde_json, existing deps), React 18 + TypeScript, Vitest, Node (fake ACP agent in tests).

**Spec:** `docs/superpowers/specs/2026-11-06-tempa-m3-agents-design.md` (approved), main spec sections 7, 8, 11.

## Global Constraints

- `crates/` keeps MSRV 1.85, no new dependency in `tempa-core`/`tempa-cli`. New public core API needs a decision record (`task.decision_required` gate).
- No new app dependency unless a task names it. ACP client on `serde_json` (decision record in Task 2).
- ACP wire facts (verify against https://agentclientprotocol.com/protocol/schema before coding Task 2): newline-delimited JSON-RPC 2.0 on the agent's stdio; `protocolVersion: 1`; client→agent `initialize`, `session/new {cwd, mcpServers:[{name, command, args, env:[{name,value}]}]}`, `session/load`, `session/prompt {sessionId, prompt:[{type:"text",text}]}` → `{stopReason}`, notification `session/cancel {sessionId}`, `session/set_model` / `session/set_config_option`; agent→client notification `session/update {sessionId, update:{sessionUpdate, …}}` and request `session/request_permission {sessionId, toolCall:{toolCallId,title,kind,locations?}, options:[{optionId,name,kind}]}` → `{outcome:{outcome:"selected",optionId}}` or `{outcome:{outcome:"cancelled"}}`.
- Client capabilities: `fs.readTextFile=false`, `fs.writeTextFile=false`, `terminal=false`.
- Agent launch table (pinned from the ACP registry 2026-11-06): claude `npx -y @agentclientprotocol/claude-agent-acp@0.101.0`; codex `npx -y @agentclientprotocol/codex-acp@2.5.0`; gemini `npx -y @google/gemini-cli@0.67.0 --acp`; opencode `opencode acp`; cursor `cursor-agent acp`. Nothing is downloaded by the app.
- Policy: never pick an `allow_always`/`reject_always` option; never send `session/set_mode`; `protected_paths` and paths outside the session root always rejected.
- Lifecycle states in plan frontmatter: `plan`, `approved`, `implement`, `verify`, `record`.
- Worktree path: `<repo parent>/<repo name>.worktrees/<task-id>`, branch `tempa/<task-id>`.
- Host data: `<app_data_dir>/projects/<project id>/tasks/<task id>/verify.json`, `<app_data_dir>/projects/<project id>/sessions/<session file>.jsonl`.
- All user-facing text in English. Agent processes spawn via `tempa_core::command` (no console window on Windows).
- OS: macOS and Windows must work (Windows CI runs app backend tests, incl. the fake agent); Linux compiles.
- Before each commit on a branch, reinstall the CLI from that branch (`cargo install --path crates/tempa-cli --locked --force`).

## Review Focus

1. **Agent process trees:** closing a chat tab, removing a task or quitting the app ends the adapter *and* its children (`npx` → `node` → agent); nothing keeps running or holding the worktree.
2. **Stuck agents:** an agent that never answers `initialize`/`session/new`, dies mid-turn, or leaves a permission request pending when the tab closes; the UI must recover (error shown, request cancelled, no hung thread).
3. **Permission paths:** relative paths, `..`, symlinks out of the worktree, Windows backslashes and drive-letter case in `locations`; scope and protected checks must hold on the normalized path.
4. **State changing under a session:** Approve pressed mid-turn, or the plan file changed by a pull; the next permission decision uses the current state, never a cached one.
5. **Worktree edge cases:** the worktree dir already exists, the branch exists without a plan, the main checkout has uncommitted changes, removing a task with unmerged commits.

Each line has its pinning test in the owning task (marked **RF-n**).

---

### Task 1: Host MCP mode, lifecycle states, recorded Verify

**Files:** Modify `crates/tempa-core/src/task.rs` (+ `tests/task.rs`); create `app/src-tauri/src/{verify.rs,mcp_host.rs}`; modify `app/src-tauri/src/{main.rs,lib.rs}`; decision `docs/decisions/<date>-host-mcp-in-app-binary.md`.

**Interfaces:**
- Produces (core): `pub const STATES: [&str; 5]`; `task::set_state(root: &Path, state: &str) -> Result<()>` (current task from branch; unknown state → error naming STATES).
- Produces (app): `verify::Verify { at: String, ok: bool, findings: Vec<contract::FindingDto> }` (serde); `verify::run(root: &Path, file: &Path) -> Result<Verify, String>` (runs `check` with harness, writes `file` atomically, then if state ∈ {approved, implement, verify}: ok → `record`, else → `implement`); `verify::load(file: &Path) -> Option<Verify>`.
- Produces (app): `mcp_host::handle_line(root: &Path, verify_file: &Path, line: &str) -> Option<String>` (`tools/call tempa_check {harness:true}` → `verify::run`; everything else → `tempa_core::mcp::handle_line`); `mcp_host::serve(root, verify_file)` (stdin/stdout loop).
- Produces (app): binary mode `tempa-app --mcp <root> --verify <file>`; `paths::task_dir(data: &Path, project_root: &Path, task_id: &str) -> PathBuf`, `paths::sessions_dir(data, project_root) -> PathBuf` (project id = `projects::id`).

- [ ] **Step 1: Failing tests:** core `set_state_rejects_unknown_and_writes_known`; app `verify_pass_moves_to_record` (harness `ok = "exit 0"`, state approved → record, file has `ok: true`), `verify_fail_moves_back_to_implement` (`exit 1`), `verify_before_approval_keeps_state_plan`, `mcp_host_routes_harness_check_to_verify` (tools/call → file written, response `isError` false), `mcp_host_delegates_other_methods` (`tools/list` has 7 tools).
- [ ] **Step 2: Run** `cargo test --workspace` and `cargo test --manifest-path app/src-tauri/Cargo.toml verify mcp_host` → FAIL (missing items).
- [ ] **Step 3: Implement** as in Interfaces; `main.rs` parses `--mcp` before `run()`.
- [ ] **Step 4: Run** both suites → PASS; decision record; task cycle; commit `feat: host MCP mode and recorded verify`.

### Task 2: ACP connection and agent launch

**Files:** Create `app/src-tauri/src/acp/{mod.rs,agents.rs,conn.rs,session.rs}`, `app/src-tauri/tests/fake-agent.mjs`; modify `lib.rs`; decision `docs/decisions/<date>-acp-client-on-serde-json.md`.

**Interfaces:**
- Produces: `agents::AgentSpec { id, name, launch: Launch, lifecycle: bool }`, `enum Launch { Npx { package: &'static str, args: &'static [&'static str] }, Binary { cmd: &'static str, args: &'static [&'static str] } }`, `pub const AGENTS: [AgentSpec; 5]` (table in Global Constraints, `lifecycle: true` until Task 7 says otherwise); `agents::resolve(spec, path_env: Option<&str>) -> Result<(PathBuf, Vec<String>), String>` (PATH search incl. PATHEXT on Windows so `npx.cmd` resolves; error = "<name> is not installed: <hint>").
- Produces: `conn::Conn::spawn(program: &Path, args: &[String], cwd: &Path, path_env: Option<&str>, on_notify: impl Fn(&str, Value) + Send + 'static, on_request: impl Fn(&str, Value) -> Result<Value, Value> + Send + Sync + 'static) -> Result<Conn, String>`; `Conn::request(&self, method: &str, params: Value, timeout: Option<Duration>) -> Result<Value, String>`; `Conn::notify(&self, method, params)`; `Conn::stop(&self)` (kills the whole tree: Unix process group, Windows `taskkill /T /F`); Drop = stop. Agent requests run on their own thread (they may wait on the user). A dead agent fails every pending request with the last 20 stderr lines.
- Produces: `session::McpLaunch { command: PathBuf, args: Vec<String> }`; `session::Session::start(conn: Conn, cwd: &Path, mcp: &McpLaunch) -> Result<Session, String>` (`initialize` and `session/new` with 30 s timeouts); `Session::prompt(&self, text: &str) -> Result<String, String>` (stop reason); `Session::cancel(&self)`; `Session::models(&self) -> &Models`; `Session::set_model(&self, id: &str) -> Result<(), String>`; `Models { current: Option<String>, available: Vec<(String, String)>, via: ModelVia }`, `enum ModelVia { ConfigOption(String), SetModel, None }` (prefer a config option with `category: "model"`).
- Produces (Tauri): `agent_list() -> Vec<AgentDto { id, name, installed: bool, hint: Option<String>, lifecycle: bool }>`; `chat_start(root: String, cwd: String, agent: String, task: Option<String>, on_event: Channel<Value>) -> Result<u32, String>`; `chat_prompt(id, text)`, `chat_cancel(id)`, `chat_stop(id)`, `chat_set_model(id, model)`, `chat_permission(id, request: u64, option: Option<String>)`. Events: `{type:"update", update}`, `{type:"permission", request, toolCall, options}`, `{type:"turn_end", stopReason}`, `{type:"error", message}`, `{type:"exit"}`. In this task every permission request goes to the UI. `ChatManager` mirrors `PtyManager` (Exit → stop all).
- Fake agent (`node tests/fake-agent.mjs`): answers initialize/session/new (`sessionId "s1"`, two models via config option); on prompt `echo <x>` streams `agent_message_chunk "echo <x>"` and a `plan` update; `edit <path>` sends `request_permission` (kind `edit`, location `<path>`, options allow_once/allow_always/reject_once) and replies with the chosen optionId as a message; `mcp` replies with the received `mcpServers` JSON; `hang` never ends the turn until `session/cancel` (→ `cancelled`); `crash` writes to stderr and exits 3; with `FAKE_AUTH=1` `session/new` fails with code -32000 `auth_required` and message "run fake login"; `spawn-child` starts a grandchild `node -e "setInterval(()=>{},1000)"` and prints its pid.

- [ ] **Step 1: Failing tests** (skip with a message if `node` is missing locally; CI has it): `handshake_prompt_streams_updates`, `mcp_server_is_injected`, `permission_request_reaches_handler_and_reply_returns`, `cancel_ends_a_hanging_turn`, `crash_fails_pending_prompt_with_stderr` (**RF-2**), `initialize_timeout_is_an_error` (fake started with `FAKE_SILENT=1`) (**RF-2**), `stop_kills_the_agent_tree` (grandchild pid gone within 5 s) (**RF-1**), `resolve_reports_missing_binary_with_hint`, `auth_required_surfaces_agent_message` (event `error` carries "run fake login"), `models_prefer_config_option`.
- [ ] **Step 2: Run** `cargo test --manifest-path app/src-tauri/Cargo.toml acp` → FAIL.
- [ ] **Step 3: Implement** per Interfaces.
- [ ] **Step 4: Run** → PASS (twice, for flakiness); clippy; task cycle; commit `feat(app): ACP client and agent launch`. If the diff exceeds 800 lines, split the Tauri commands into their own PR.

### Task 3: Chat tab

**Files:** Modify `app/src/layout/{tabs.ts,Workspace.tsx}`, `app/src/lib/ipc.ts`; create `app/src/chat/{Chat.tsx,reducer.ts,AgentPicker.tsx}` + tests; backend appends every chat event to the session JSONL (`paths::sessions_dir`, file `<unix ms>-<agent>.jsonl`).

**Interfaces:**
- Consumes: Task 2 commands and events.
- Produces: tab kind `{ kind: "terminal" } | { kind: "chat", agent: string, task: string | null, cwd: string }` in `tabs.ts` (action `newChat`); `reducer.ts`: `type ChatState = { items: Item[]; plan: PlanEntry[]; pending: Permission[]; busy: boolean }`, `applyEvent(s: ChatState, e: ChatEvent): ChatState` (agent chunks append to the open agent message; `tool_call` adds a card; `tool_call_update` merges by `toolCallId`; `plan` replaces entries; `permission` queues; `turn_end` clears busy); `AgentPicker` lists `agent_list()` (disabled + hint when not installed, "context-only" tag when `lifecycle` false).

- [ ] **Step 1: Failing tests:** `reducer: chunks join into one message`, `tool_call_update merges by id`, `plan update replaces entries`, `permission queues and answer removes it`; `Chat: Stop sends chat_cancel`, `Allow once sends chat_permission with that optionId`; `AgentPicker: not installed is disabled with hint`; tabs `newChat adds a chat tab`.
- [ ] **Step 2: Run** `cd app && bun run test` → FAIL.
- [ ] **Step 3: Implement**; model `<select>` shown only when `models.available` is non-empty.
- [ ] **Step 4: Run** → PASS; `bun run check`; commit `feat(app): chat tab`.

### Task 4: Tool policy, permission prompt, preamble

**Files:** Create `app/src-tauri/src/acp/policy.rs`; modify `acp/` chat wiring; core: make the protected-path matcher used by `tempa check` public if it is not (`check::is_protected(cfg: &Config, rel: &str) -> bool`).

**Interfaces:**
- Produces: `enum Phase { Explore, Plan, Approved }` (Approved = approved/implement/verify/record); `enum Verdict { Allow, Deny(String), Ask }`; `policy::decide(phase: Phase, kind: &str, paths: &[String], scope: &Path, cfg: &Config) -> Verdict` with the spec's ordered rules (normalize each path: join relative to `scope`, resolve `..`, canonicalize the deepest existing ancestor, compare case-insensitively on Windows; edit/delete/move with no location → Ask); `policy::pick(options: &[Value], v: &Verdict) -> Option<String>` (Allow → `allow_once`, Deny → `reject_once`, never `*_always`; no matching option → None = Ask).
- Produces: `policy::preamble(task: Option<(&str, &str)>) -> String` (task id + state, the five rules, the seven `tempa_*` tool names), prepended to the first prompt of a session.
- Behavior: the phase is read from the plan file on **every** permission request (**RF-4**); Deny shows a card "Rejected: <reason>" in the chat; Ask sends the `permission` event; when the session stops, pending requests answer `cancelled` (**RF-2**). First prompt while state is `approved` → `set_state("implement")`.

- [ ] **Step 1: Failing tests:** `policy_table` (Plan×edit→Deny, Plan×read→Allow, Plan×execute→Ask, Approved×edit inside→Allow, Approved×edit protected→Deny, Explore×edit→Deny, edit without location→Ask), `paths_outside_scope_are_denied` (`../x`, absolute elsewhere, symlink pointing out, `C:\` case on Windows) (**RF-3**), `pick_never_allows_always`, `preamble_names_task_and_tools`; fake-agent: `edit_before_approval_is_rejected_then_allowed_after_approve` (approve between two prompts in the same session) (**RF-4**), `pending_permission_is_cancelled_on_stop` (**RF-2**).
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** → PASS; decision record if core API changed; commit `feat(app): tool policy and permission prompt`.

### Task 5: Tasks and worktrees

**Files:** Modify `crates/tempa-core/src/{gitx.rs,task.rs}` (+ tests); create `app/src-tauri/src/tasks.rs`; create `app/src/task/{TaskPanel.tsx,TaskPanel.test.tsx}`; modify `app/src/{App.tsx,sidebar/Sidebar.tsx,lib/ipc.ts}`; decision for the core API.

**Interfaces:**
- Produces (core): `gitx::worktree_add(root, path: &Path, branch: &str, create: bool) -> Result<()>`, `gitx::worktree_remove(root, path) -> Result<()>`, `gitx::worktrees(root) -> Result<Vec<(PathBuf, Option<String>)>>`, `gitx::branches(root, prefix: &str) -> Result<Vec<String>>`, `gitx::show(root, rev: &str, path: &str) -> Result<String>`, `gitx::is_merged(root, branch: &str, into: &str) -> bool`; `task::task_new_in(root: &Path, slug: &str, worktrees: &Path) -> Result<(String, PathBuf)>` (creates `tempa/<id>` from HEAD in a new worktree under `worktrees`, writes the plan there; the main checkout is untouched).
- Produces (app): `tasks::TaskDto { id, state, worktree: Option<String>, approved_by: Option<String>, changes_entry: bool, verify: Option<verify::Verify> }`; `tasks::worktrees_dir(root) -> PathBuf`; `tasks::list(root, data) -> Vec<TaskDto>` (branches `tempa/*` with a plan, state read from the worktree file or `git show`); `tasks::create(root, data, slug) -> Result<TaskDto, String>`; `tasks::open(root, data, id)` (adds a worktree for an existing branch); `tasks::remove(root, id) -> Result<bool, String>` (true = branch deleted; unmerged → worktree removed only if clean, branch kept, message says so); `tasks::approve(worktree)` = `task_approve`; Tauri commands `task_list/create/open/remove/approve/verify/plan` (verify = `verify::run` in-process).
- UI: tasks under the active project in the sidebar (state badge), "+ New task" (slug input), Task tab in the right panel (Task | Contract) with plan preview, Approve (only in `plan`, after the plan passes the contract), Verify, last harness result, worktree path, "Open terminal here", "Chat"; the watcher also runs for each open worktree.

- [ ] **Step 1: Failing tests:** core `task_new_in_leaves_main_checkout_alone`, `worktree_add_existing_branch`, `branches_lists_task_branches`; app `create_then_list_shows_plan_state`, `open_existing_branch_adds_worktree`, `create_fails_cleanly_when_dir_exists` (**RF-5**), `branch_without_plan_is_not_listed` (**RF-5**), `create_with_dirty_main_checkout_works` (**RF-5**), `remove_unmerged_keeps_branch` (**RF-5**); frontend `TaskPanel: Approve calls task_approve and hides`, `Verify shows the result`, `Sidebar lists tasks with state`.
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** all suites → PASS; decision record; commit `feat(app): tasks and worktrees`.

### Task 6: Lifecycle end to end, PlanPanel, Explore, history

**Files:** Create `app/src-tauri/tests/lifecycle.rs` (integration test, uses `CARGO_BIN_EXE_tempa-app` as the injected MCP command); extend `tests/fake-agent.mjs` with `mcp-call <tool> <json>` (spawns the received MCP command, calls the tool, replies with the result); modify chat/task UI.

**Interfaces:**
- Consumes: Tasks 1–5.
- Produces: PlanPanel in the Task tab from the chat's `plan` entries (status icons pending/in_progress/completed); sidebar "Explore" opens a chat in the project root with `task = null` (Phase::Explore); opening a task's chat shows the latest transcript of that task read-only above the new session (`chat_history(root, task) -> Vec<Value>`); `session/load` used when the agent advertised `loadSession` and the transcript has its session id.

- [ ] **Step 1: Failing tests:** `lifecycle.rs::plan_approve_implement_verify_record` (create task → fake `mcp-call tempa_plan` fills the plan → edit rejected → approve → edit allowed → `mcp-call tempa_check {"harness":true}` → state `record`, `verify.json` ok → `mcp-call tempa_log_change` → `changes_entry` true); `explore_rejects_edits`; frontend `PlanPanel renders entries`, `Explore opens a chat without task`, `history renders read-only`.
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** → PASS (also on Windows CI); commit `feat(app): lifecycle end to end`.

### Task 7: Terminal banner, agent matrix, context-only

**Files:** Modify `app/src-tauri/src/pty.rs` (foreground process name, Unix: `MasterPty::process_group_leader` + `ps -o comm= -p`), `app/src/terminal/Terminal.tsx`; create `docs/agent-matrix.md`; core `targets::acp_lifecycle(agent: Agent) -> bool` used by `tempa doctor` and `AGENTS[].lifecycle`.

**Interfaces:**
- Produces: `PtyManager::foreground(id) -> Option<String>` (None on Windows); event `{type:"agent", name}` sent at most every 2 s while output flows; banner "This <name> session runs outside the tempa lifecycle; CI still checks the contract." with a "Chat in tempa instead" action that opens a chat tab for the same agent.
- `docs/agent-matrix.md`: one row per agent (adapter version, MCP injected ✓/✗, permission before edit ✓/✗, notes on modes/model selection), filled by the owner; `acp_lifecycle` returns false for rows marked ✗.

- [ ] **Step 1: Failing tests:** `foreground_reports_sleep` (Unix: spawn `sh -c "exec sleep 5"` → `Some("sleep")`), `banner shows for known agent names`, core `doctor_labels_context_only_agents`.
- [ ] **Step 2: Run** → FAIL. **Step 3: Implement.** **Step 4: Run** → PASS; commit `feat(app): terminal agent banner and agent matrix`.

Final whole-branch review with a fresh reviewer after Task 7, Review Focus above; then the owner fills `docs/agent-matrix.md` with real agents.
