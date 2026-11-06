//! tempa desktop shell: terminals per project and the contract panel, on tempa-core.

mod contract;
mod env;
mod projects;
mod pty;
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
async fn contract_run(root: String, action: String) -> Result<Vec<String>, String> {
    contract::run(std::path::Path::new(&root), &action)
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

#[tauri::command]
fn pty_kill(ptys: State<'_, PtyManager>, id: u32) -> Result<(), String> {
    ptys.kill(id)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(PtyManager::default())
        .manage(Watches::default())
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
            watch_stop
        ])
        .build(tauri::generate_context!())
        .expect("tempa app failed to start")
        .run(|app, event| {
            // No orphan shells when the app quits.
            if let RunEvent::Exit = event {
                app.state::<PtyManager>().kill_all();
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
