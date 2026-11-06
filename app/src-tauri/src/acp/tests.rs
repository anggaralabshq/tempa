//! Against `tests/fake-agent.mjs` (needs `node`; CI has it).

use super::conn::{Conn, Handlers};
use super::session::{McpLaunch, ModelVia, Session, models};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// The fake agent runs on Node (CI has it; so does anyone running npx adapters).
fn node() -> PathBuf {
    PathBuf::from("node")
}

fn fake_args() -> Vec<String> {
    vec![concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fake-agent.mjs").to_string()]
}

struct Fake {
    conn: Conn,
    notes: mpsc::Receiver<(String, Value)>,
    exited: mpsc::Receiver<String>,
}

fn spawn(env: &[(&str, &str)], answer: Option<&'static str>) -> Fake {
    let (ntx, notes) = mpsc::channel();
    let (etx, exited) = mpsc::channel();
    let handlers = Handlers {
        notify: Box::new(move |m: &str, p: Value| {
            let _ = ntx.send((m.to_string(), p));
        }),
        request: Box::new(move |m: &str, _p: Value| match (m, answer) {
            ("session/request_permission", Some(opt)) => {
                Ok(json!({"outcome": {"outcome": "selected", "optionId": opt}}))
            }
            _ => Err(json!({"code": -32601, "message": "not supported"})),
        }),
        exit: Box::new(move |tail: String| {
            let _ = etx.send(tail);
        }),
    };
    let env: Vec<(String, String)> = env
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let conn = Conn::spawn(
        &node(),
        &fake_args(),
        Path::new(env!("CARGO_MANIFEST_DIR")),
        None,
        &env,
        handlers,
    )
    .unwrap();
    Fake {
        conn,
        notes,
        exited,
    }
}

fn mcp() -> McpLaunch {
    McpLaunch {
        command: PathBuf::from("/bin/tempa-app"),
        args: vec!["--mcp".into(), "/r".into()],
    }
}

fn start(
    f: Fake,
) -> (
    Session,
    mpsc::Receiver<(String, Value)>,
    mpsc::Receiver<String>,
) {
    let s = Session::start(
        f.conn,
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &mcp(),
        Duration::from_secs(20),
    )
    .unwrap();
    (s, f.notes, f.exited)
}

fn texts(notes: &mpsc::Receiver<(String, Value)>) -> Vec<String> {
    notes
        .try_iter()
        .filter(|(m, _)| m == "session/update")
        .filter_map(|(_, p)| p["update"]["content"]["text"].as_str().map(String::from))
        .collect()
}

#[test]
fn handshake_prompt_streams_updates() {
    let (s, notes, _) = start(spawn(&[], None));
    assert_eq!(s.id(), "s1");
    assert_eq!(s.prompt("echo hi").unwrap(), "end_turn");
    std::thread::sleep(Duration::from_millis(100));
    let all: Vec<(String, Value)> = notes.try_iter().collect();
    assert!(
        all.iter()
            .any(|(_, p)| p["update"]["content"]["text"] == "echo hi"),
        "{all:?}"
    );
    assert!(
        all.iter()
            .any(|(_, p)| p["update"]["sessionUpdate"] == "plan")
    );
}

#[test]
fn mcp_server_is_injected() {
    let (s, notes, _) = start(spawn(&[], None));
    s.prompt("mcp").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let got: Value = serde_json::from_str(&texts(&notes)[0]).unwrap();
    assert_eq!(got[0]["name"], "tempa");
    assert_eq!(got[0]["command"], "/bin/tempa-app");
    assert_eq!(got[0]["args"], json!(["--mcp", "/r"]));
}

#[test]
fn permission_request_reaches_handler_and_reply_returns() {
    let (s, notes, _) = start(spawn(&[], Some("a1")));
    s.prompt("edit src/a.ts").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let got: Value = serde_json::from_str(&texts(&notes)[0]).unwrap();
    assert_eq!(got, json!({"outcome": "selected", "optionId": "a1"}));
}

#[test]
fn cancel_ends_a_hanging_turn() {
    let (s, _, _) = start(spawn(&[], None));
    let s = std::sync::Arc::new(s);
    let s2 = s.clone();
    let t = std::thread::spawn(move || s2.prompt("hang"));
    std::thread::sleep(Duration::from_millis(300));
    s.cancel();
    assert_eq!(t.join().unwrap().unwrap(), "cancelled");
}

#[test]
fn crash_fails_pending_prompt_with_stderr() {
    let (s, _, exited) = start(spawn(&[], None));
    let err = s.prompt("crash").unwrap_err();
    assert!(err.contains("fake agent crashed"), "{err}");
    assert!(
        exited
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .contains("fake agent crashed")
    );
}

#[test]
fn initialize_timeout_is_an_error() {
    let f = spawn(&[("FAKE_SILENT", "1")], None);
    let t = Instant::now();
    let err = Session::start(f.conn, Path::new("."), &mcp(), Duration::from_millis(500))
        .err()
        .unwrap();
    assert!(
        err.contains("initialize") && err.contains("timed out"),
        "{err}"
    );
    assert!(t.elapsed() < Duration::from_secs(5));
}

#[test]
fn auth_required_surfaces_agent_message() {
    let f = spawn(&[("FAKE_AUTH", "1")], None);
    let err = Session::start(f.conn, Path::new("."), &mcp(), Duration::from_secs(20))
        .err()
        .unwrap();
    assert!(err.contains("run fake login"), "{err}");
}

fn alive(pid: u32) -> bool {
    if cfg!(windows) {
        let out = tempa_core::command("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).contains(&pid.to_string())
    } else {
        tempa_core::command("kill")
            .args(["-0", &pid.to_string()])
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap()
            .success()
    }
}

#[test]
fn stop_kills_the_agent_tree() {
    let (s, notes, _) = start(spawn(&[], None));
    s.prompt("spawn-child").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let pid: u32 = texts(&notes)[0].parse().unwrap();
    assert!(alive(pid));
    drop(s);
    let t = Instant::now();
    while alive(pid) && t.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(!alive(pid), "grandchild {pid} outlived the session");
}

#[test]
fn models_prefer_config_option() {
    let new = json!({"sessionId": "s", "configOptions": [
        {"id": "mode", "name": "Mode", "type": "select", "currentValue": "a", "options": [{"value": "a", "name": "A"}]},
        {"id": "m", "name": "Model", "category": "model", "type": "select", "currentValue": "x",
         "options": [{"group": "g", "name": "G", "options": [{"value": "x", "name": "X"}, {"value": "y", "name": "Y"}]}]}],
        "models": {"currentModelId": "old", "availableModels": [{"modelId": "old", "name": "Old"}]}});
    let m = models(&new);
    assert_eq!(m.current.as_deref(), Some("x"));
    assert_eq!(
        m.available,
        [
            ("x".to_string(), "X".to_string()),
            ("y".to_string(), "Y".to_string())
        ]
    );
    assert!(matches!(m.via, ModelVia::ConfigOption(ref id) if id == "m"));
    let legacy = models(
        &json!({"models": {"currentModelId": "old", "availableModels": [{"modelId": "old", "name": "Old"}]}}),
    );
    assert!(matches!(legacy.via, ModelVia::SetModel));
    assert!(matches!(models(&json!({})).via, ModelVia::None));
}

