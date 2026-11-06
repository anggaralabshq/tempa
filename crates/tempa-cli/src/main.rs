use clap::{Parser, Subcommand};
use std::path::Path;
use std::process::{Command, ExitCode};
use tempa_core::{Finding, Severity, check, import, init, sync, task};

#[derive(Parser)]
#[command(name = "tempa", version, about = "Satu kontrak konteks untuk semua coding agent")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Buat tempa.toml, dokumen kontrak, git hooks, dan dokumen serah-terima DevOps
    Init,
    /// Generate file konteks tiap agent dari AGENTS.md dan .tempa/
    Sync {
        /// Hanya periksa drift, tidak menulis (untuk CI)
        #[arg(long, conflicts_with = "fix")]
        check: bool,
        /// Untuk hook pre-commit: sync + git add bila aman, blokir bila target diedit langsung
        #[arg(long)]
        fix: bool,
    },
    /// Periksa drift, kontrak dokumen, aturan task, lalu harness
    Check {
        /// Lewati harness (untuk step Bitbucket Pipelines)
        #[arg(long)]
        contract_only: bool,
        /// Branch dasar (default: BITBUCKET_PR_DESTINATION_BRANCH atau [ci].base)
        #[arg(long)]
        base: Option<String>,
    },
    /// Kelola task di luar aplikasi
    Task {
        #[command(subcommand)]
        cmd: TaskCmd,
    },
    /// Buat decision record baru dari template
    Decide { slug: String },
    /// Tambah entri changes untuk task saat ini
    Log {
        #[arg(long)]
        summary: String,
        #[arg(long)]
        reason: String,
        /// Path decision terkait
        #[arg(long)]
        decision: Option<String>,
        /// lulus | gagal | belum
        #[arg(long, default_value = "belum")]
        harness: String,
        /// Agent dan model, mis. "codex / gpt-5"
        #[arg(long)]
        agent: String,
    },
    /// Cetak body PR untuk ditempel ke Bitbucket
    PrBody,
    /// Serap .claude/, CLAUDE.md, .cursorrules, dll. ke sumber netral, lalu sync
    Import {
        /// Tulis perubahan (tanpa flag ini hanya laporan)
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum TaskCmd {
    /// Buat branch tempa/<id> dan file plan
    New { slug: String },
    /// Tandai plan disetujui (panduan proses)
    Approve,
}

fn print_import(r: &import::ImportReport) {
    for p in &r.initialized {
        println!("created {p}");
    }
    for p in &r.written {
        println!("ditulis: {p}");
    }
    for p in &r.merged_from {
        println!("digabung dari: {p}");
    }
    println!("paragraf duplikat dilewati: {}", r.duplicates_skipped);
    for (n, c) in &r.harness_added {
        println!("harness ditambahkan: {n} = {c}");
    }
    for g in &r.protected_added {
        println!("protected_paths ditambahkan: {g}");
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
    let out = Command::new("git").arg("-C").arg(root).args(["add", "-A", "--"]).args(paths).output()?;
    if !out.status.success() {
        return Err(tempa_core::Error(format!("git add gagal: {}", String::from_utf8_lossy(&out.stderr))));
    }
    Ok(())
}

fn run(cmd: Cmd, root: &Path) -> tempa_core::Result<bool> {
    match cmd {
        Cmd::Sync { check: true, .. } => Ok(print_findings(&sync::drift_findings(&sync::drift(root)?))),
        Cmd::Sync { fix: true, .. } => {
            let (changed, blockers) = sync::sync_fix(root)?;
            if !blockers.is_empty() {
                return Ok(print_findings(&sync::drift_findings(&blockers)));
            }
            git_add(root, &changed)?;
            for p in &changed {
                println!("synced {p}");
            }
            Ok(false)
        }
        Cmd::Check { contract_only, base } => {
            Ok(print_findings(&check::check(root, &check::CheckOpts { contract_only, base })?))
        }
        Cmd::Task { cmd: TaskCmd::New { slug } } => {
            let id = task::task_new(root, &slug)?;
            println!("task {id} dibuat di branch tempa/{id}; isi plan lalu jalankan tempa task approve");
            Ok(false)
        }
        Cmd::Task { cmd: TaskCmd::Approve } => {
            task::task_approve(root)?;
            println!("plan disetujui");
            Ok(false)
        }
        Cmd::Decide { slug } => {
            println!("{}", task::decide(root, &slug)?);
            Ok(false)
        }
        Cmd::Log { summary, reason, decision, harness, agent } => {
            task::log(root, &task::LogEntry { summary, reason, decision, harness, agent })?;
            println!("entri changes ditambahkan");
            Ok(false)
        }
        Cmd::Init => {
            for p in init::init(root)? {
                println!("created {p}");
            }
            for p in sync::sync(root)? {
                println!("synced {p}");
            }
            println!("Serahkan docs/tempa-devops.md ke tim DevOps.");
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
                println!("Jalankan ulang dengan --yes untuk menulis.");
                return Ok(false);
            }
            for p in sync::sync(root)? {
                println!("synced {p}");
            }
            Ok(false)
        }
        Cmd::Sync { .. } => {
            for p in sync::sync(root)? {
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
