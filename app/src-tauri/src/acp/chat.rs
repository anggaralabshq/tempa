//! Live chat sessions for the UI: one agent process per chat, events out through
//! `emit`, permission requests waiting on the user's answer.

use super::conn::{Conn, Handlers};
use super::policy::{self, Phase, Verdict};
use super::session::{McpLaunch, Session};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

pub type Emit = Arc<dyn Fn(Value) + Send + Sync>;
type Waiting = Arc<Mutex<HashMap<u64, mpsc::Sender<Option<String>>>>>;

struct Chat {
    session: Session,
    waiting: Waiting,
    emit: Emit,
    log: Log,
    cwd: PathBuf,
    task: Option<String>,
    first: AtomicBool,
}

/// Append-only JSONL transcript of a chat (None = not recorded).
#[derive(Clone, Default)]
struct Log(Option<Arc<Mutex<std::fs::File>>>);

impl Log {
    fn open(path: Option<PathBuf>) -> Result<Log, String> {
        let Some(path) = path else {
            return Ok(Log(None));
        };
        // Transcripts hold code and command output: owner-only on Unix.
        let mut dir = std::fs::DirBuilder::new();
        let mut file = std::fs::OpenOptions::new();
        file.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
            dir.mode(0o700);
            file.mode(0o600);
        }
        if let Some(d) = path.parent() {
            dir.recursive(true).create(d).map_err(|e| e.to_string())?;
        }
        let f = file
            .open(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Log(Some(Arc::new(Mutex::new(f)))))
    }

    fn write(&self, v: &Value) {
        if let Some(f) = &self.0 {
            let mut f = f.lock().unwrap_or_else(|p| p.into_inner());
            // One write per line: a reader never sees half an event.
            let _ = f.write_all(format!("{v}\n").as_bytes());
        }
    }
}

impl Chat {
    /// Every open permission request answers `cancelled` (ACP requires it on cancel).
    fn cancel_waiting(&self) {
        for (_, tx) in self
            .waiting
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .drain()
        {
            let _ = tx.send(None);
        }
    }
}

/// How to run the agent process.
pub struct Run<'a> {
    pub program: &'a Path,
    pub args: &'a [String],
    pub cwd: &'a Path,
    pub path_env: Option<&'a str>,
    /// The task this chat works on (its worktree is `cwd`); None = Explore.
    pub task: Option<&'a str>,
}

/// The host's verdict on a permission request, from the phase on disk now (RF-4).
fn verdict(cwd: &Path, explore: bool, call: &Value) -> Verdict {
    let Ok(cfg) = tempa_core::config::Config::load(cwd) else {
        return Verdict::Ask;
    };
    let phase = if explore {
        Phase::Explore
    } else {
        policy::phase(crate::verify::current_state(cwd).as_deref())
    };
    let paths: Vec<String> = call["locations"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|l| l["path"].as_str().map(String::from))
        .collect();
    policy::decide(
        phase,
        call["kind"].as_str().unwrap_or("other"),
        &paths,
        cwd,
        &cfg,
    )
}

#[derive(Debug, Clone, Serialize)]
pub struct Started {
    pub id: u32,
    pub current_model: Option<String>,
    /// (id, name).
    pub models: Vec<(String, String)>,
}

#[derive(Default)]
pub struct ChatManager {
    next: AtomicU32,
    chats: Mutex<HashMap<u32, Arc<Chat>>>,
}

/// First `npx -y` run downloads the adapter.
const INIT_TIMEOUT: Duration = Duration::from_secs(120);

impl ChatManager {
    fn get(&self, id: u32) -> Result<Arc<Chat>, String> {
        self.chats
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&id)
            .cloned()
            .ok_or_else(|| "chat has ended".into())
    }

    /// Start `program args` in `cwd` as an ACP agent with the tempa MCP server.
    /// Events: `update`, `permission`, `turn_end`, `error`, `exit`.
    pub fn start_with(
        &self,
        run: Run,
        mcp: McpLaunch,
        transcript: Option<PathBuf>,
        emit: Emit,
    ) -> Result<Started, String> {
        let Run {
            program,
            args,
            cwd,
            path_env,
            task,
        } = run;
        let log = Log::open(transcript)?;
        let (raw, l) = (emit, log.clone());
        let emit: Emit = Arc::new(move |e: Value| {
            l.write(&e);
            raw(e);
        });
        let waiting: Waiting = Arc::default();
        let next_req = Arc::new(AtomicU64::new(1));
        let (on_update, on_ask, on_exit) = (emit.clone(), emit.clone(), emit.clone());
        let w = Arc::clone(&waiting);
        let (scope, explore) = (cwd.to_path_buf(), task.is_none());
        let handlers = Handlers {
            notify: Box::new(move |method: &str, p: Value| {
                if method == "session/update" {
                    on_update(json!({"type": "update", "update": p["update"]}));
                }
            }),
            request: Box::new(move |method: &str, p: Value| {
                if method != "session/request_permission" {
                    return Err(
                        json!({"code": -32601, "message": format!("{method} is not supported by tempa")}),
                    );
                }
                let v = verdict(&scope, explore, &p["toolCall"]);
                let options = p["options"].as_array().cloned().unwrap_or_default();
                let auto = match &v {
                    Verdict::Deny(reason) => {
                        Some(("rejected", reason.clone(), policy::pick(&options, &v)))
                    }
                    Verdict::Allow => {
                        policy::pick(&options, &v).map(|o| ("allowed", String::new(), Some(o)))
                    }
                    Verdict::Ask => None,
                };
                if let Some((verdict, reason, option)) = auto {
                    on_ask(
                        json!({"type": "auto", "verdict": verdict, "reason": reason, "toolCall": p["toolCall"]}),
                    );
                    return Ok(match option {
                        Some(o) => json!({"outcome": {"outcome": "selected", "optionId": o}}),
                        None => json!({"outcome": {"outcome": "cancelled"}}),
                    });
                }
                let request = next_req.fetch_add(1, Ordering::Relaxed);
                let (tx, rx) = mpsc::channel();
                w.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .insert(request, tx);
                on_ask(
                    json!({"type": "permission", "request": request, "toolCall": p["toolCall"], "options": p["options"]}),
                );
                Ok(match rx.recv().ok().flatten() {
                    Some(option) => json!({"outcome": {"outcome": "selected", "optionId": option}}),
                    None => json!({"outcome": {"outcome": "cancelled"}}),
                })
            }),
            exit: Box::new(move |tail: String| {
                on_exit(json!({"type": "exit", "stderr": tail}));
            }),
        };
        let conn = Conn::spawn(program, args, cwd, path_env, &[], handlers)?;
        let session = Session::start(conn, cwd, &mcp, INIT_TIMEOUT)?;
        let started = Started {
            id: self.next.fetch_add(1, Ordering::Relaxed) + 1,
            current_model: session.models().current.clone(),
            models: session.models().available.clone(),
        };
        let chat = Arc::new(Chat {
            session,
            waiting,
            emit,
            log,
            cwd: cwd.to_path_buf(),
            task: task.map(String::from),
            first: AtomicBool::new(true),
        });
        self.chats
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(started.id, chat);
        Ok(started)
    }

    /// Send a turn; the result arrives as `turn_end` or `error`.
    pub fn prompt(&self, id: u32, text: &str) -> Result<(), String> {
        let chat = self.get(id)?;
        chat.log.write(&json!({"type": "user", "text": text}));
        let mut text = text.to_string();
        if chat.first.swap(false, Ordering::Relaxed) {
            let mut state = crate::verify::current_state(&chat.cwd);
            if state.as_deref() == Some("approved")
                && tempa_core::task::set_state(&chat.cwd, "implement").is_ok()
            {
                state = Some("implement".into());
            }
            let task = chat
                .task
                .as_deref()
                .map(|t| (t, state.as_deref().unwrap_or("plan")));
            text = policy::preamble(task) + &text;
        }
        std::thread::spawn(move || {
            let ev = match chat.session.prompt(&text) {
                Ok(reason) => json!({"type": "turn_end", "stopReason": reason}),
                Err(message) => json!({"type": "error", "message": message}),
            };
            (chat.emit)(ev);
        });
        Ok(())
    }

    pub fn cancel(&self, id: u32) {
        if let Ok(chat) = self.get(id) {
            chat.cancel_waiting();
            chat.session.cancel();
        }
    }

    pub fn answer(&self, id: u32, request: u64, option: Option<String>) -> Result<(), String> {
        let chat = self.get(id)?;
        let tx = chat
            .waiting
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&request)
            .ok_or("this permission request was already answered")?;
        let _ = tx.send(option);
        Ok(())
    }

    pub fn set_model(&self, id: u32, model: &str) -> Result<(), String> {
        self.get(id)?.session.set_model(model)
    }

    /// End the chat: cancel the turn, answer what waits, stop the agent tree.
    pub fn stop(&self, id: u32) {
        let chat = self
            .chats
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&id);
        if let Some(chat) = chat {
            chat.cancel_waiting();
            chat.session.cancel();
            // Not on the last Arc drop: a running prompt thread still holds one.
            chat.session.stop();
        }
    }

    pub fn stop_all(&self) {
        let ids: Vec<u32> = self
            .chats
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .keys()
            .copied()
            .collect();
        for id in ids {
            self.stop(id);
        }
    }
}
