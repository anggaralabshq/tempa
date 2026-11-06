use clap::{Parser, Subcommand};
use std::path::Path;
use std::process::{Command, ExitCode};
use tempa_core::{Finding, Severity, check, doctor, import, init, sync, task};

#[derive(Parser)]
#[command(
    name = "tempa",
    version,
    about = "One context contract and workflow for every coding agent"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create tempa.toml, contract docs, git hooks and the DevOps handoff doc
    Init,
    /// Per-agent parity report (always exits 0)
    Doctor,
    /// Generate each agent's context files from AGENTS.md and .tempa/
    Sync {
        /// Only check for drift, write nothing (for CI)
        #[arg(long, conflicts_with = "fix")]
        check: bool,
        /// For the pre-commit hook: sync + git add when safe, block when a target was edited directly
        #[arg(long)]
        fix: bool,
        /// Overwrite targets that were edited directly or not written by tempa
        #[arg(long, conflicts_with_all = ["check", "fix"])]
        force: bool,
    },
    /// Check drift, contract docs, task rules, then the harness
    Check {
        /// Skip the harness (for a CI step)
        #[arg(long)]
        contract_only: bool,
        /// Base branch (default: BITBUCKET_PR_DESTINATION_BRANCH or [ci].base)
        #[arg(long)]
        base: Option<String>,
    },
    /// Manage tasks from the command line
    Task {
        #[command(subcommand)]
        cmd: TaskCmd,
    },
    /// Create a decision record from the template
    Decide { slug: String },
    /// Add a changes entry for the current task
    Log {
        #[arg(long)]
        summary: String,
        #[arg(long)]
        reason: String,
        /// Path of the related decision
        #[arg(long)]
        decision: Option<String>,
        /// pass | fail | pending
        #[arg(long, default_value = "pending")]
        harness: String,
        /// Agent and model, e.g. "codex / gpt-5"
        #[arg(long)]
        agent: String,
    },
    /// Print the pull request body
    PrBody,
    /// Run the MCP server on stdio (agents start this themselves)
    Mcp,
    /// Absorb .claude/, CLAUDE.md, .cursorrules, etc. into the neutral source, then sync
    Import {
        /// Write changes (without this flag, report only)
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum TaskCmd {
    /// Create branch tempa/<id> and its plan file
    New { slug: String },
    /// Mark the plan approved (a human step)
    Approve,
}

fn warn_hooks_path(root: &Path) {
    if let Some(p) = tempa_core::gitx::get_config(root, "core.hooksPath") {
        if p != init::HOOKS_DIR {
            println!(
                "warning: core.hooksPath = {p}, left unchanged. Call `tempa sync --fix` from its pre-commit hook and `tempa check` from its pre-push hook."
            );
        }
    }
}

fn print_doctor(r: &doctor::DoctorReport) {
    let w = r
        .rows
        .iter()
        .map(|row| row.protected.len())
        .chain(["protected".len()])
        .max()
        .unwrap_or(0);
    println!(
        "{:<10} {:>5} {:>5}  {:<w$}  lifecycle",
        "agent", "files", "gaps", "protected"
    );
    for row in &r.rows {
        println!(
            "{:<10} {:>5} {:>5}  {:<w$}  {}",
            row.agent.name(),
            row.files,
            row.gaps.len(),
            row.protected,
            row.lifecycle
        );
    }
    for g in r.rows.iter().flat_map(|row| &row.gaps) {
        println!(
            "- {} {}: {} (served via tempa_context; enforced by tempa check)",
            g.agent.name(),
            g.kind,
            g.name
        );
    }
    for l in &r.legacy {
        println!("legacy: {l} (safe to delete once reviewed)");
    }
    if !r.hooks_ok {
        println!("warning: git hooks are not installed; run tempa init");
    }
    println!("drift: {}", r.drift);
}

fn print_import(r: &import::ImportReport) {
    for p in &r.initialized {
        println!("created {p}");
    }
    for p in &r.written {
        println!("write: {p}");
    }
    for p in &r.merged_from {
        println!("merged from: {p}");
    }
    println!("duplicate paragraphs skipped: {}", r.duplicates_skipped);
    for (n, c) in &r.harness_added {
        println!("harness added: {n} = {c}");
    }
    for g in &r.protected_added {
        println!("protected_paths added: {g}");
    }
    for g in &r.gaps {
        println!("gap: {g}");
    }
}

/// Print findings; return whether any is an error.
fn print_findings(findings: &[Finding]) -> bool {
    let (mut e, mut w) = (0, 0);
    for f in findings {
        let level = match f.severity {
            Severity::Error => {
                e += 1;
                "error"
            }
            Severity::Warning => {
                w += 1;
                "warning"
            }
        };
        match &f.path {
            Some(p) => println!("{level}[{}] {p}: {}", f.code, f.message),
            None => println!("{level}[{}] {}", f.code, f.message),
        }
    }
    println!("{e} error, {w} warning");
    e > 0
}

fn git_add(root: &Path, paths: &[String]) -> tempa_core::Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["add", "-A", "--"])
        .args(paths)
        .output()?;
    if !out.status.success() {
        return Err(tempa_core::Error(format!(
            "git add failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(())
}

/// stdio JSON-RPC loop. stdout carries replies only; diagnostics go to stderr.
fn serve_mcp() -> tempa_core::Result<()> {
    use std::io::{BufRead, Write};
    let root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| std::path::PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let mut out = std::io::stdout().lock();
    for line in std::io::stdin().lock().lines() {
        if let Some(reply) = tempa_core::mcp::handle_line(&root, &line?) {
            writeln!(out, "{reply}")?;
            out.flush()?;
        }
    }
    Ok(())
}

fn run(cmd: Cmd, root: &Path) -> tempa_core::Result<bool> {
    match cmd {
        Cmd::Sync { check: true, .. } => {
            Ok(print_findings(&sync::drift_findings(&sync::drift(root)?)))
        }
        Cmd::Sync { fix: true, .. } => {
            let (changed, blockers) = sync::sync_fix(root)?;
            if !blockers.is_empty() {
                return Ok(print_findings(&blockers));
            }
            git_add(root, &changed)?;
            for p in &changed {
                println!("synced {p}");
            }
            Ok(false)
        }
        Cmd::Check {
            contract_only,
            base,
        } => Ok(print_findings(&check::check(
            root,
            &check::CheckOpts {
                contract_only,
                base,
            },
        )?)),
        Cmd::Task {
            cmd: TaskCmd::New { slug },
        } => {
            let id = task::task_new(root, &slug)?;
            println!(
                "task {id} created on branch tempa/{id}; fill in the plan, then run tempa task approve"
            );
            Ok(false)
        }
        Cmd::Task {
            cmd: TaskCmd::Approve,
        } => {
            task::task_approve(root)?;
            println!("plan approved");
            Ok(false)
        }
        Cmd::Decide { slug } => {
            println!("{}", task::decide(root, &slug)?);
            Ok(false)
        }
        Cmd::Log {
            summary,
            reason,
            decision,
            harness,
            agent,
        } => {
            task::log(
                root,
                &task::LogEntry {
                    summary,
                    reason,
                    decision,
                    harness,
                    agent,
                },
            )?;
            println!("changes entry added");
            Ok(false)
        }
        Cmd::Init => {
            if let Some(p) = init::existing_agent_context(root) {
                return Err(tempa_core::Error(format!(
                    "this repo already has agent context ({p}); run `tempa import --yes` instead (import also does what init does)"
                )));
            }
            for p in init::init(root)? {
                println!("created {p}");
            }
            for p in sync::sync(root)? {
                println!("synced {p}");
            }
            warn_hooks_path(root);
            println!(
                "Hand docs/tempa-devops.md to whoever owns CI when you are ready to enforce in CI."
            );
            Ok(false)
        }
        Cmd::Doctor => {
            print_doctor(&doctor::doctor(root)?);
            Ok(false)
        }
        Cmd::Mcp => {
            serve_mcp()?;
            Ok(false)
        }
        Cmd::PrBody => {
            print!("{}", task::pr_body(root)?);
            Ok(false)
        }
        Cmd::Import { yes } => {
            let report = import::import(root, yes)?;
            print_import(&report);
            if !yes {
                println!("Run again with --yes to write.");
                return Ok(false);
            }
            for p in &report.synced {
                println!("synced {p}");
            }
            warn_hooks_path(root);
            for p in &report.conflicts {
                eprintln!(
                    "error[sync.conflict] {p}: edit could not be absorbed; move it into its source, then run tempa sync --force"
                );
            }
            Ok(!report.conflicts.is_empty())
        }
        Cmd::Sync { force, .. } => {
            let changed = if force {
                sync::sync_force(root)?
            } else {
                sync::sync(root)?
            };
            for p in changed {
                println!("synced {p}");
            }
            Ok(false)
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let Some(cmd) = cli.cmd else {
        use clap::CommandFactory;
        Cli::command().print_help().ok();
        return ExitCode::from(2);
    };
    match run(cmd, Path::new(".")) {
        Ok(false) => ExitCode::SUCCESS,
        Ok(true) => ExitCode::from(1),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}
