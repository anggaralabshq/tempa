//! tempa desktop shell: terminals per project and the contract panel, on tempa-core.

mod acp;
mod contract;
mod env;
mod mcp_host;
mod projects;
mod pty;
mod verify;
mod watcher;

use pty::{CommandBuilder, PtyManager};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::ipc::{Channel, Response};
use tauri::{Emitter, Manager, RunEvent, State};

/// One watcher per open project, keyed by root. Dropping a `Watch` stops it.
#[derive(Default)]
struct Watches(Mutex<HashMap<String, watcher::Watch>>);

#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn home_dir() -> Option<std::ffi::OsString> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
}

/// Start the user's shell in `cwd` (home when absent). Output goes to `on_data` as raw
/// bytes; `on_exit` gets the exit code. Async so the first PATH probe never blocks the UI.
#[tauri::command]
async fn pty_spawn(
    ptys: State<'_, PtyManager>,
    cwd: Option<String>,
    cols: u16,
    rows: u16,
    on_data: Channel<Response>,
    on_exit: Channel<i32>,
) -> Result<u32, String> {
    let mut cmd = CommandBuilder::new(env::user_shell());
    if !cfg!(windows) {
        cmd.arg("-l");
    }
    if let Some(dir) = cwd.map(std::ffi::OsString::from).or_else(home_dir) {
        cmd.cwd(dir);
    }
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    if let Some(path) = env::terminal_path() {
        cmd.env("PATH", path);
    }
    ptys.spawn(
        cmd,
        cols,
        rows,
        move |bytes| {
            let _ = on_data.send(Response::new(bytes));
        },
        move |code| {
            let _ = on_exit.send(code);
        },
    )
}

#[tauri::command]
fn pty_write(ptys: State<'_, PtyManager>, id: u32, data: String) -> Result<(), String> {
    ptys.write(id, data.as_bytes())
}

#[tauri::command]
fn pty_resize(ptys: State<'_, PtyManager>, id: u32, cols: u16, rows: u16) -> Result<(), String> {
    ptys.resize(id, cols, rows)
}

fn projects_file(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("projects.json"))
}

#[tauri::command]
fn projects_list(app: tauri::AppHandle) -> Result<Vec<projects::Project>, String> {
    Ok(projects::list(&projects_file(&app)?))
}

#[tauri::command]
fn projects_add(app: tauri::AppHandle, path: String) -> Result<projects::Project, String> {
    projects::add(&projects_file(&app)?, std::path::Path::new(&path))
}

#[tauri::command]
fn projects_remove(app: tauri::AppHandle, id: String) -> Result<(), String> {
    projects::remove(&projects_file(&app)?, &id)
}

#[tauri::command]
async fn contract_status(root: String) -> contract::ContractStatus {
    let path = env::terminal_path().or_else(|| std::env::var("PATH").ok());
    contract::status(std::path::Path::new(&root), path.as_deref())
}

#[tauri::command]
async fn contract_preview(root: String, action: String) -> Result<Vec<String>, String> {
    contract::preview(std::path::Path::new(&root), &action)
}

#[tauri::command]
async fn contract_run(
    root: String,
    action: String,
    reviewed: Vec<String>,
) -> Result<Vec<String>, String> {
    contract::run(std::path::Path::new(&root), &action, &reviewed)
}

/// Watch `root`: source edits sync, drift changes emit `contract-changed` with an `Update`.
#[tauri::command]
fn watch_start(
    app: tauri::AppHandle,
    watches: State<'_, Watches>,
    root: String,
) -> Result<(), String> {
    let key = root.clone();
    let w = watcher::watch(std::path::Path::new(&root), move |mut u| {
        u.root.clone_from(&key); // the frontend's key, not the canonical path
        let _ = app.emit("contract-changed", u);
    })?;
    watches.0.lock().unwrap().insert(root, w);
    Ok(())
}

#[tauri::command]
fn watch_stop(watches: State<'_, Watches>, root: String) {
    watches.0.lock().unwrap().remove(&root);
}

/// Last host-recorded harness run of `task` in project `root`.
#[tauri::command]
fn verify_result(
    app: tauri::AppHandle,
    root: String,
    task: String,
) -> Result<Option<verify::Verify>, String> {
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(verify::load(&verify::file(&data, &root, &task)))
}

#[derive(serde::Serialize)]
struct AgentDto {
    id: &'static str,
    name: &'static str,
    installed: bool,
    hint: Option<String>,
    lifecycle: bool,
}

#[tauri::command]
async fn agent_list() -> Vec<AgentDto> {
    let path = env::terminal_path();
    acp::agents::AGENTS
        .iter()
        .map(|a| {
            let r = acp::agents::resolve(a, path.as_deref());
            AgentDto {
                id: a.id,
                name: a.name,
                installed: r.is_ok(),
                hint: r.err(),
                lifecycle: a.lifecycle,
            }
        })
        .collect()
}

/// Start `agent` in `cwd` (a task worktree, or the project root to explore) with the
/// tempa MCP server injected; events arrive on `on_event`.
#[tauri::command]
async fn chat_start(
    app: tauri::AppHandle,
    chats: State<'_, acp::chat::ChatManager>,
    root: String,
    cwd: String,
    agent: String,
    task: Option<String>,
    on_event: Channel<serde_json::Value>,
) -> Result<acp::chat::Started, String> {
    let spec = acp::agents::find(&agent).ok_or("unknown agent")?;
    let path = env::terminal_path();
    let (program, args) = acp::agents::resolve(spec, path.as_deref())?;
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let verify = verify::file(&data, &root, task.as_deref().unwrap_or("_explore"));
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mcp = acp::session::McpLaunch {
        command: exe,
        args: vec![
            "--mcp".into(),
            cwd.clone(),
            "--verify".into(),
            verify.to_string_lossy().into(),
        ],
    };
    let emit: acp::chat::Emit = std::sync::Arc::new(move |e| {
        let _ = on_event.send(e);
    });
    chats.start_with(
        &program,
        &args,
        std::path::Path::new(&cwd),
        path.as_deref(),
        mcp,
        emit,
    )
}

#[tauri::command]
fn chat_prompt(
    chats: State<'_, acp::chat::ChatManager>,
    id: u32,
    text: String,
) -> Result<(), String> {
    chats.prompt(id, &text)
}

#[tauri::command]
fn chat_cancel(chats: State<'_, acp::chat::ChatManager>, id: u32) {
    chats.cancel(id)
}

#[tauri::command]
fn chat_stop(chats: State<'_, acp::chat::ChatManager>, id: u32) {
    chats.stop(id)
}

#[tauri::command]
async fn chat_set_model(
    chats: State<'_, acp::chat::ChatManager>,
    id: u32,
    model: String,
) -> Result<(), String> {
    chats.set_model(id, &model)
}

#[tauri::command]
fn chat_permission(
    chats: State<'_, acp::chat::ChatManager>,
    id: u32,
    request: u64,
    option: Option<String>,
) -> Result<(), String> {
    chats.answer(id, request, option)
}

#[tauri::command]
fn pty_kill(ptys: State<'_, PtyManager>, id: u32) -> Result<(), String> {
    ptys.kill(id)
}

/// `tempa-app --mcp <root> --verify <file>`: serve the injected tempa MCP server and
/// exit. Returns false when the arguments are not that mode.
pub fn mcp_mode(args: &[String]) -> bool {
    match args {
        [_, mcp, root, verify, file] if mcp == "--mcp" && verify == "--verify" => {
            mcp_host::serve(std::path::Path::new(root), std::path::Path::new(file));
            true
        }
        _ => false,
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(PtyManager::default())
        .manage(Watches::default())
        .manage(acp::chat::ChatManager::default())
        .invoke_handler(tauri::generate_handler![
            app_version,
            pty_spawn,
            pty_write,
            pty_resize,
            pty_kill,
            projects_list,
            projects_add,
            projects_remove,
            contract_status,
            contract_preview,
            contract_run,
            watch_start,
            watch_stop,
            verify_result,
            agent_list,
            chat_start,
            chat_prompt,
            chat_cancel,
            chat_stop,
            chat_set_model,
            chat_permission
        ])
        .build(tauri::generate_context!())
        .expect("tempa app failed to start")
        .run(|app, event| {
            // No orphan shells when the app quits.
            if let RunEvent::Exit = event {
                app.state::<PtyManager>().kill_all();
                app.state::<acp::chat::ChatManager>().stop_all();
            }
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn app_version_matches_package() {
        assert_eq!(super::app_version(), "0.1.0");
    }
}
