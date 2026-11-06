//! ACP client: run coding agents as subprocesses and talk to them over stdio.

pub mod agents;
pub mod chat;
pub mod conn;
pub mod session;

#[cfg(test)]
mod tests;
