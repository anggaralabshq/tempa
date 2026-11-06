//! One pseudo-terminal per pane: spawn, write, resize, kill.

pub use portable_pty::CommandBuilder;
use portable_pty::{ChildKiller, MasterPty, PtySize, native_pty_system};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, mpsc};

struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
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
        let writer = pair.master.take_writer().map_err(err)?;
        let killer = child.clone_killer();
        let id = self.next.fetch_add(1, Ordering::Relaxed) + 1;
        self.sessions().insert(
            id,
            Session {
                master: pair.master,
                writer,
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

    pub fn write(&self, id: u32, data: &[u8]) -> Result<(), String> {
        let mut s = self.sessions();
        let w = &mut s.get_mut(&id).ok_or("terminal has exited")?.writer;
        w.write_all(data).and_then(|_| w.flush()).map_err(err)
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

    /// Ask the process to end; the waiter removes the session once it has.
    pub fn kill(&self, id: u32) -> Result<(), String> {
        match self.sessions().get_mut(&id) {
            Some(s) => s.killer.kill().map_err(err),
            None => Ok(()),
        }
    }

    pub fn kill_all(&self) {
        for s in self.sessions().values_mut() {
            let _ = s.killer.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    fn collect(
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

    #[test]
    fn spawn_echo_reports_output_and_exit() {
        let mut cmd = if cfg!(windows) {
            CommandBuilder::new("cmd")
        } else {
            CommandBuilder::new("sh")
        };
        if cfg!(windows) {
            cmd.args(["/C", "echo hi & exit 3"]);
        } else {
            cmd.args(["-c", "echo hi; exit 3"]);
        }
        let (_m, _id, drx, erx) = collect(cmd);
        assert_eq!(erx.recv_timeout(Duration::from_secs(10)).unwrap(), 3);
        let out: Vec<u8> = drx.try_iter().flatten().collect();
        assert!(String::from_utf8_lossy(&out).contains("hi"), "{out:?}");
    }

    #[test]
    fn kill_ends_process() {
        let mut cmd = if cfg!(windows) {
            CommandBuilder::new("cmd")
        } else {
            CommandBuilder::new("sh")
        };
        if cfg!(windows) {
            cmd.args(["/C", "ping -n 30 127.0.0.1 >NUL"]);
        } else {
            cmd.args(["-c", "sleep 30"]);
        }
        let (m, id, _drx, erx) = collect(cmd);
        m.kill(id).unwrap();
        erx.recv_timeout(Duration::from_secs(10))
            .expect("exit after kill");
        assert!(m.write(id, b"x").is_err(), "killed session is gone");
    }

    #[test]
    fn write_reaches_the_shell() {
        let mut cmd = if cfg!(windows) {
            CommandBuilder::new("cmd")
        } else {
            CommandBuilder::new("sh")
        };
        if !cfg!(windows) {
            cmd.args(["-c", "read line; echo got:$line"]);
        }
        let (m, id, drx, erx) = collect(cmd);
        m.write(
            id,
            if cfg!(windows) {
                b"echo got:ok\r\nexit\r\n"
            } else {
                b"ok\n"
            },
        )
        .unwrap();
        erx.recv_timeout(Duration::from_secs(10)).unwrap();
        let out: Vec<u8> = drx.try_iter().flatten().collect();
        assert!(
            String::from_utf8_lossy(&out).contains("got:ok"),
            "{}",
            String::from_utf8_lossy(&out)
        );
    }
}
