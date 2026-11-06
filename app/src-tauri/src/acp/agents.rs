//! The five v1 agents and how to launch their ACP adapters (pinned from the ACP
//! registry, cdn.agentclientprotocol.com, 2026-11-06). Nothing is downloaded by tempa.

use std::path::PathBuf;

pub enum Launch {
    /// `npx -y <package> <args>`: needs Node's npx on PATH.
    Npx {
        package: &'static str,
        args: &'static [&'static str],
    },
    /// A CLI already installed on PATH.
    Binary {
        cmd: &'static str,
        args: &'static [&'static str],
    },
}

pub struct AgentSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub launch: Launch,
    /// How to install it when missing.
    pub install: &'static str,
    /// Passed the lifecycle-parity check (docs/agent-matrix.md).
    pub lifecycle: bool,
}

pub const AGENTS: [AgentSpec; 5] = [
    AgentSpec {
        id: "claude",
        name: "Claude Code",
        launch: Launch::Npx {
            package: "@agentclientprotocol/claude-agent-acp@0.101.0",
            args: &[],
        },
        install: "install Node.js (npx), then log in with `claude`",
        lifecycle: true,
    },
    AgentSpec {
        id: "codex",
        name: "Codex",
        launch: Launch::Npx {
            package: "@agentclientprotocol/codex-acp@2.5.0",
            args: &[],
        },
        install: "install Node.js (npx), then log in with `codex login`",
        lifecycle: true,
    },
    AgentSpec {
        id: "gemini",
        name: "Gemini CLI",
        launch: Launch::Npx {
            package: "@google/gemini-cli@0.67.0",
            args: &["--acp"],
        },
        install: "install Node.js (npx), then log in with `gemini`",
        lifecycle: true,
    },
    AgentSpec {
        id: "opencode",
        name: "OpenCode",
        launch: Launch::Binary {
            cmd: "opencode",
            args: &["acp"],
        },
        install: "install OpenCode (https://opencode.ai)",
        lifecycle: true,
    },
    AgentSpec {
        id: "cursor",
        name: "Cursor",
        launch: Launch::Binary {
            cmd: "cursor-agent",
            args: &["acp"],
        },
        install: "install the Cursor CLI (https://cursor.com/cli)",
        lifecycle: true,
    },
];

pub fn find(id: &str) -> Option<&'static AgentSpec> {
    AGENTS.iter().find(|a| a.id == id)
}

/// `name` on `path_env` (the process PATH when None); PATHEXT on Windows, so `npx`
/// finds `npx.cmd`.
pub fn which(name: &str, path_env: Option<&str>) -> Option<PathBuf> {
    let path = path_env
        .map(std::ffi::OsString::from)
        .or_else(|| std::env::var_os("PATH"))?;
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into())
            .split(';')
            .map(String::from)
            .collect()
    } else {
        vec![String::new()]
    };
    std::env::split_paths(&path).find_map(|dir| {
        exts.iter()
            .map(|e| dir.join(format!("{name}{e}")))
            .find(|p| p.is_file())
    })
}

/// Program and arguments for `spec`, or why it cannot run here.
pub fn resolve(spec: &AgentSpec, path_env: Option<&str>) -> Result<(PathBuf, Vec<String>), String> {
    let missing = || format!("{} is not installed: {}", spec.name, spec.install);
    match &spec.launch {
        Launch::Npx { package, args } => {
            let npx = which("npx", path_env).ok_or_else(missing)?;
            let mut all = vec!["-y".to_string(), package.to_string()];
            all.extend(args.iter().map(|a| a.to_string()));
            Ok((npx, all))
        }
        Launch::Binary { cmd, args } => Ok((
            which(cmd, path_env).ok_or_else(missing)?,
            args.iter().map(|a| a.to_string()).collect(),
        )),
    }
}
