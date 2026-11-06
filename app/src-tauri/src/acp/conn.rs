//! Newline-delimited JSON-RPC 2.0 with an agent subprocess, both directions.

use serde_json::{Value, json};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

type Reply = Result<Value, String>;
pub type OnNotify = Box<dyn Fn(&str, Value) + Send>;
pub type OnRequest = Box<dyn Fn(&str, Value) -> Result<Value, Value> + Send + Sync>;

pub struct Handlers {
    /// Agent → client notification (`session/update`).
    pub notify: OnNotify,
    /// Agent → client request; runs on its own thread, so it may wait on the user.
    /// `Err` is a JSON-RPC error object.
    pub request: OnRequest,
    /// The agent's stdout closed; gets the last stderr lines.
    pub exit: Box<dyn FnOnce(String) + Send>,
}

struct Inner {
    stdin: Mutex<ChildStdin>,
    pending: Mutex<HashMap<u64, mpsc::Sender<Reply>>>,
    next: AtomicU64,
    child: Mutex<Child>,
    /// Set once the agent is gone: why.
    dead: Mutex<Option<String>>,
}

impl Inner {
    fn send(&self, msg: &Value) -> Result<(), String> {
        let mut w = self.stdin.lock().unwrap_or_else(|p| p.into_inner());
        writeln!(w, "{msg}")
            .and_then(|_| w.flush())
            .map_err(|e| format!("agent is not running: {e}"))
    }
}

/// Owns the agent process; dropping it ends the process tree.
pub struct Conn(Arc<Inner>);

const STDERR_LINES: usize = 20;

impl Conn {
    pub fn spawn(
        program: &Path,
        args: &[String],
        cwd: &Path,
        path_env: Option<&str>,
        env: &[(String, String)],
        h: Handlers,
    ) -> Result<Conn, String> {
        let mut cmd = tempa_core::command(&program.to_string_lossy());
        cmd.args(args)
            .current_dir(cwd)
            .envs(env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(p) = path_env {
            cmd.env("PATH", p);
        }
        #[cfg(unix)]
        {
            // Own process group, so stop() reaches npx → node → agent.
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("could not start {}: {e}", program.display()))?;
        let stdin = child.stdin.take().expect("piped");
        let stdout = child.stdout.take().expect("piped");
        let stderr = child.stderr.take().expect("piped");
        let inner = Arc::new(Inner {
            stdin: Mutex::new(stdin),
            pending: Mutex::new(HashMap::new()),
            next: AtomicU64::new(1),
            child: Mutex::new(child),
            dead: Mutex::new(None),
        });

        let tail = Arc::new(Mutex::new(VecDeque::new()));
        let (err_done_tx, err_done) = mpsc::channel::<()>();
        let t = Arc::clone(&tail);
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let mut t = t.lock().unwrap_or_else(|p| p.into_inner());
                if t.len() == STDERR_LINES {
                    t.pop_front();
                }
                t.push_back(line);
            }
            let _ = err_done_tx.send(());
        });

        let me = Arc::clone(&inner);
        let request = Arc::new(h.request);
        let notify = h.notify;
        let exit = h.exit;
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let Ok(msg) = serde_json::from_str::<Value>(&line) else {
                    continue; // ponytail: adapters sometimes log to stdout; skip non-JSON.
                };
                match (msg.get("method").and_then(Value::as_str), msg.get("id")) {
                    (Some(method), Some(id)) => {
                        let (me, request) = (Arc::clone(&me), Arc::clone(&request));
                        let (method, id) = (method.to_string(), id.clone());
                        std::thread::spawn(move || {
                            let reply = match request(&method, msg["params"].clone()) {
                                Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
                                Err(error) => json!({"jsonrpc": "2.0", "id": id, "error": error}),
                            };
                            let _ = me.send(&reply);
                        });
                    }
                    (Some(method), None) => notify(method, msg["params"].clone()),
                    (None, Some(id)) => {
                        let tx = id.as_u64().and_then(|id| {
                            me.pending
                                .lock()
                                .unwrap_or_else(|p| p.into_inner())
                                .remove(&id)
                        });
                        if let Some(tx) = tx {
                            let reply = match msg.get("error") {
                                Some(e) => {
                                    Err(e["message"].as_str().unwrap_or("agent error").to_string())
                                }
                                None => Ok(msg["result"].clone()),
                            };
                            let _ = tx.send(reply);
                        }
                    }
                    (None, None) => {}
                }
            }
            // Give stderr a moment to flush the reason the agent died.
            let _ = err_done.recv_timeout(Duration::from_secs(1));
            let tail: Vec<String> = tail
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .iter()
                .cloned()
                .collect();
            let why = format!("agent exited: {}", tail.join("\n"));
            *me.dead.lock().unwrap_or_else(|p| p.into_inner()) = Some(why.clone());
            for (_, tx) in me.pending.lock().unwrap_or_else(|p| p.into_inner()).drain() {
                let _ = tx.send(Err(why.clone()));
            }
            exit(tail.join("\n"));
        });
        Ok(Conn(inner))
    }

    /// Call `method` and wait for its result (`timeout` None = no limit).
    pub fn request(
        &self,
        method: &str,
        params: Value,
        timeout: Option<Duration>,
    ) -> Result<Value, String> {
        let id = self.0.next.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        {
            let dead = self.0.dead.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(why) = dead.as_ref() {
                return Err(why.clone());
            }
            self.0
                .pending
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .insert(id, tx);
        }
        self.0
            .send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))?;
        match timeout {
            Some(t) => rx.recv_timeout(t).map_err(|_| {
                self.0
                    .pending
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .remove(&id);
                format!("{method} timed out after {} s", t.as_secs_f32())
            })?,
            None => rx
                .recv()
                .map_err(|_| "agent connection closed".to_string())?,
        }
    }

    pub fn notify(&self, method: &str, params: Value) -> Result<(), String> {
        self.0
            .send(&json!({"jsonrpc": "2.0", "method": method, "params": params}))
    }

    /// End the agent and everything it started.
    pub fn stop(&self) {
        let mut child = self.0.child.lock().unwrap_or_else(|p| p.into_inner());
        let pid = child.id().to_string();
        if cfg!(windows) {
            let _ = tempa_core::command("taskkill")
                .args(["/T", "/F", "/PID", &pid])
                .output();
        } else {
            let _ = tempa_core::command("kill")
                .args(["-KILL", "--", &format!("-{pid}")])
                .output();
        }
        let _ = child.kill();
        let _ = child.wait();
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        self.stop();
    }
}
