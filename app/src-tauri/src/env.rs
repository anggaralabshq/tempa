//! PATH for shells started by a GUI app. Adapted from Termul
//! (src-tauri/src/pty/env_refresh.rs, MIT); see THIRD_PARTY_NOTICES.md.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const MARKER: &str = "__TEMPA_LOGIN_PATH__=";
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Merge PATH lists: `first` order, then segments of `second` not seen yet.
/// `;` lists (Windows) dedupe case-insensitively.
pub fn merge_path_segments(first: &str, second: &str, delimiter: char) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut out: Vec<&str> = Vec::new();
    for seg in first.split(delimiter).chain(second.split(delimiter)) {
        let seg = seg.trim();
        let key = if delimiter == ';' {
            seg.to_ascii_lowercase()
        } else {
            seg.to_string()
        };
        if !seg.is_empty() && seen.insert(key) {
            out.push(seg);
        }
    }
    out.join(&delimiter.to_string())
}

/// PATH after the last probe marker.
fn parse_login_path_output(output: &[u8]) -> Option<String> {
    let output = String::from_utf8_lossy(output);
    let (_, path) = output.rsplit_once(MARKER)?;
    let path = path.lines().next()?.trim();
    (!path.is_empty()).then(|| path.to_string())
}

/// The user's shell for new terminals.
pub fn user_shell() -> String {
    static SHELL: OnceLock<String> = OnceLock::new();
    SHELL.get_or_init(detect_shell).clone()
}

fn detect_shell() -> String {
    if cfg!(windows) {
        // ponytail: pwsh when installed, else Windows PowerShell (always present).
        let has_pwsh = tempa_core::command("pwsh")
            .args(["-NoLogo", "-NoProfile", "-Command", "exit"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        return if has_pwsh {
            "pwsh.exe"
        } else {
            "powershell.exe"
        }
        .into();
    }
    std::env::var("SHELL")
        .ok()
        .filter(|s| s.starts_with('/'))
        .unwrap_or_else(|| {
            if cfg!(target_os = "macos") {
                "/bin/zsh"
            } else {
                "/bin/bash"
            }
            .into()
        })
}

/// PATH for terminals: the interactive login shell's PATH (macOS/Linux GUI launches
/// do not inherit it) merged with this process's PATH. Probed once.
pub fn terminal_path() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let inherited = std::env::var("PATH").unwrap_or_default();
            // ponytail: Windows uses the PATH inherited from Explorer; re-reading the
            // registry (Termul does) only matters when PATH changed after login.
            if cfg!(windows) {
                return None;
            }
            let login = probe_login_path(&user_shell())?;
            Some(merge_path_segments(&login, &inherited, ':'))
        })
        .clone()
}

fn probe_login_path(shell: &str) -> Option<String> {
    let name = std::path::Path::new(shell).file_name()?.to_str()?;
    let script = match name {
        "bash" | "zsh" => format!("printf '\\n{MARKER}%s\\n' \"$PATH\""),
        "fish" => format!("printf '\\n{MARKER}%s\\n' (string join : $PATH)"),
        _ => return None,
    };
    let mut child = Command::new(shell)
        .args(["-ilc", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let deadline = Instant::now() + PROBE_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
            _ => {
                // A startup file that waits for input must not hang the app.
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
    parse_login_path_output(&reader.join().ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_path_segments_dedupes_and_keeps_order() {
        assert_eq!(merge_path_segments("/a:/b", "/b:/c::/a", ':'), "/a:/b:/c");
        assert_eq!(
            merge_path_segments(r"C:\X;c:\y", r"C:\Y;D:\z", ';'),
            r"C:\X;c:\y;D:\z"
        );
    }

    #[cfg(unix)]
    #[test]
    fn probe_reads_path_from_a_login_shell() {
        let p = probe_login_path("/bin/bash").or_else(|| probe_login_path("/bin/zsh"));
        assert!(p.is_some_and(|p| p.contains("/usr/bin")));
    }

    #[test]
    fn parse_login_path_output_takes_last_marker() {
        let out =
            b"motd\n__TEMPA_LOGIN_PATH__=/old\nbanner\n__TEMPA_LOGIN_PATH__=/usr/bin:/opt/bin\n";
        assert_eq!(
            parse_login_path_output(out).as_deref(),
            Some("/usr/bin:/opt/bin")
        );
        assert_eq!(parse_login_path_output(b"no marker"), None);
        assert_eq!(parse_login_path_output(b"__TEMPA_LOGIN_PATH__=\n"), None);
    }
}
