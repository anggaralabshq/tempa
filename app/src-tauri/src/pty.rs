//! One pseudo-terminal per pane: spawn, write, resize, kill.

pub use portable_pty::CommandBuilder;
use portable_pty::{ChildKiller, MasterPty, PtySize, native_pty_system};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, mpsc};

struct Session {
    master: Box<dyn MasterPty + Send>,
    /// Input queue for the session's writer thread: a write blocked on a full tty holds
    /// up neither the UI nor the other terminals, and keystrokes stay in order.
    input: mpsc::Sender<Vec<u8>>,
    killer: Box<dyn ChildKiller + Send + Sync>,
}

/// Live terminals. A session leaves the map when its process exits.
#[derive(Default)]
pub struct PtyManager {
    next: AtomicU32,
    sessions: Arc<Mutex<HashMap<u32, Session>>>,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

impl PtyManager {
    fn sessions(&self) -> std::sync::MutexGuard<'_, HashMap<u32, Session>> {
        self.sessions.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Start `cmd` on a new pty. `on_data` gets output chunks; `on_exit` runs once,
    /// after the last chunk, with the exit code.
    pub fn spawn(
        &self,
        cmd: CommandBuilder,
        cols: u16,
        rows: u16,
        on_data: impl Fn(Vec<u8>) + Send + 'static,
        on_exit: impl FnOnce(i32) + Send + 'static,
    ) -> Result<u32, String> {
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = native_pty_system().openpty(size).map_err(err)?;
        let mut child = pair.slave.spawn_command(cmd).map_err(err)?;
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().map_err(err)?;
        let mut writer = pair.master.take_writer().map_err(err)?;
        let (input, queue) = mpsc::channel::<Vec<u8>>();
        // Ends when the session (and so `input`) is dropped.
        std::thread::spawn(move || {
            while let Ok(bytes) = queue.recv() {
                if writer
                    .write_all(&bytes)
                    .and_then(|_| writer.flush())
                    .is_err()
                {
                    break;
                }
            }
        });
        let killer = child.clone_killer();
        let id = self.next.fetch_add(1, Ordering::Relaxed) + 1;
        self.sessions().insert(
            id,
            Session {
                master: pair.master,
                input,
                killer,
            },
        );

        // Waiter: when the process exits, drop the session (closes the master, which
        // is what ends the reader on Windows ConPTY).
        let (code_tx, code_rx) = mpsc::channel();
        let sessions = Arc::clone(&self.sessions);
        std::thread::spawn(move || {
            let code = child.wait().map(|s| s.exit_code() as i32).unwrap_or(-1);
            let _ = code_tx.send(code);
            sessions
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(&id);
        });
        std::thread::spawn(move || {
            let mut buf = [0u8; 16 * 1024];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                on_data(buf[..n].to_vec());
            }
            on_exit(code_rx.recv().unwrap_or(-1));
        });
        Ok(id)
    }

    /// Queue `data` for the terminal; never blocks.
    pub fn write(&self, id: u32, data: &[u8]) -> Result<(), String> {
        let s = self.sessions();
        let session = s.get(&id).ok_or("terminal has exited")?;
        session
            .input
            .send(data.to_vec())
            .map_err(|_| "terminal has exited".to_string())
    }

    pub fn resize(&self, id: u32, cols: u16, rows: u16) -> Result<(), String> {
        let s = self.sessions();
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        s.get(&id)
            .ok_or("terminal has exited")?
            .master
            .resize(size)
            .map_err(err)
    }

    /// End the terminal: kill the shell, then close the pty. Closing the master is what
    /// ends the rest of the tree (SIGHUP to the foreground group on Unix; on Windows every
    /// process attached to the pseudoconsole), so agents started in the shell do not
    /// outlive their pane.
    pub fn kill(&self, id: u32) -> Result<(), String> {
        let session = self.sessions().remove(&id);
        if let Some(mut s) = session {
            // ponytail: ignore the result; portable-pty 0.9 on Windows can report a
            // successful TerminateProcess as an error, and closing the pty follows anyway.
            let _ = s.killer.kill();
        }
        Ok(())
    }

    pub fn kill_all(&self) {
        let all: Vec<Session> = self.sessions().drain().map(|(_, s)| s).collect();
        for mut s in all {
            let _ = s.killer.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn shell(script_unix: &str, script_win: &str) -> CommandBuilder {
        if cfg!(windows) {
            let mut c = CommandBuilder::new("cmd");
            c.args(["/C", script_win]);
            c
        } else {
            let mut c = CommandBuilder::new("sh");
            c.args(["-c", script_unix]);
            c
        }
    }

    fn start(
        cmd: CommandBuilder,
    ) -> (
        PtyManager,
        u32,
        mpsc::Receiver<Vec<u8>>,
        mpsc::Receiver<i32>,
    ) {
        let m = PtyManager::default();
        let (dtx, drx) = mpsc::channel();
        let (etx, erx) = mpsc::channel();
        let id = m
            .spawn(
                cmd,
                80,
                24,
                move |b| {
                    let _ = dtx.send(b);
                },
                move |c| {
                    let _ = etx.send(c);
                },
            )
            .unwrap();
        (m, id, drx, erx)
    }

    /// Output and exit code. Answers ConPTY's cursor-position request (ESC[6n) the way
    /// xterm.js does in the app; without an answer ConPTY holds the output back.
    fn until_exit(
        m: &PtyManager,
        id: u32,
        drx: &mpsc::Receiver<Vec<u8>>,
        erx: &mpsc::Receiver<i32>,
    ) -> (String, i32) {
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut out = Vec::new();
        let mut answered = false;
        loop {
            if let Ok(b) = drx.recv_timeout(Duration::from_millis(50)) {
                out.extend(b);
                if !answered && out.windows(4).any(|w| w == b"\x1b[6n") {
                    let _ = m.write(id, b"\x1b[1;1R");
                    answered = true;
                }
            }
            if let Ok(code) = erx.try_recv() {
                out.extend(drx.try_iter().flatten());
                return (String::from_utf8_lossy(&out).to_string(), code);
            }
            assert!(
                Instant::now() < deadline,
                "no exit; output so far: {:?}",
                String::from_utf8_lossy(&out)
            );
        }
    }

    #[test]
    fn spawn_echo_reports_output_and_exit() {
        let (m, id, drx, erx) = start(shell("echo hi; exit 3", "echo hi & exit 3"));
        let (out, code) = until_exit(&m, id, &drx, &erx);
        assert_eq!(code, 3, "{out}");
        assert!(out.contains("hi"), "{out}");
    }

    #[test]
    fn write_reaches_the_shell() {
        let (m, id, drx, erx) = start(shell(
            "read line; echo got:$line",
            "set /p line= & call echo got:%line%",
        ));
        m.write(id, b"ok\r").unwrap();
        let (out, _) = until_exit(&m, id, &drx, &erx);
        assert!(out.contains("got:ok"), "{out}");
    }

    #[test]
    fn kill_closes_the_session_and_ends_the_process() {
        let (m, id, drx, erx) = start(shell("sleep 30", "ping -n 30 127.0.0.1 >NUL"));
        m.kill(id).unwrap();
        assert!(m.write(id, b"x").is_err(), "session closes at once");
        until_exit(&m, id, &drx, &erx);
    }

    #[cfg(unix)]
    #[test]
    fn blocked_write_does_not_freeze_other_terminals() {
        let (m, busy, _d, _e) = start(shell("sleep 30", ""));
        let m = std::sync::Arc::new(m);
        let other = m
            .spawn(shell("sleep 30", ""), 80, 24, |_| {}, |_| {})
            .unwrap();
        // A big paste into a shell whose foreground job never reads: the tty fills up.
        let m2 = std::sync::Arc::clone(&m);
        std::thread::spawn(move || {
            let _ = m2.write(busy, "x\n".repeat(200_000).as_bytes());
        });
        std::thread::sleep(Duration::from_millis(500));
        let (tx, rx) = mpsc::channel();
        let m3 = std::sync::Arc::clone(&m);
        std::thread::spawn(move || {
            let _ = tx.send(m3.resize(other, 100, 30));
        });
        let resized = rx.recv_timeout(Duration::from_secs(2));
        m.kill_all();
        assert!(resized.is_ok(), "resize waited on another pane's write");
    }
}
