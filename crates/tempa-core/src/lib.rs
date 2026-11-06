//! Core logic for tempa: config, import, sync, contract checks.

pub mod check;
pub mod config;
pub mod contract;
pub mod date;
pub mod doctor;
pub mod gitx;
pub mod import;
pub mod init;
pub mod md;
pub mod source;
pub mod sync;
pub mod targets;
pub mod task;

use std::fmt;

/// Error with a human-readable (Indonesian) message.
#[derive(Debug)]
pub struct Error(pub String);

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

macro_rules! from_err {
    ($($t:ty),*) => {$(
        impl From<$t> for Error {
            fn from(e: $t) -> Self { Error(e.to_string()) }
        }
    )*};
}
from_err!(std::io::Error, toml::de::Error, toml_edit::TomlError, serde_json::Error);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// One problem reported by a check. `code` is stable and machine-readable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub code: &'static str,
    pub severity: Severity,
    pub path: Option<String>,
    pub message: String,
}

impl Finding {
    pub fn error(code: &'static str, path: Option<String>, message: impl Into<String>) -> Self {
        Finding { code, severity: Severity::Error, path, message: message.into() }
    }

    pub fn warning(code: &'static str, path: Option<String>, message: impl Into<String>) -> Self {
        Finding { code, severity: Severity::Warning, path, message: message.into() }
    }
}

/// Write via `<path>.tempa-tmp` + rename so a crash never leaves a half file.
pub fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tempa-tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
