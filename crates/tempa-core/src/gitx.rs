//! Thin wrappers over the `git` CLI (no libgit2).

use crate::config::Config;
use crate::{Error, Result};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["-c", "core.quotepath=false"])
        .args(args)
        .output()
        .map_err(|e| Error(format!("could not run git: {e}")))?;
    if !out.status.success() {
        return Err(Error(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn is_repo(root: &Path) -> bool {
    git(root, &["rev-parse", "--is-inside-work-tree"]).is_ok_and(|s| s.trim() == "true")
}

/// Branch name; `BITBUCKET_BRANCH` wins (Pipelines checks out a detached HEAD).
pub fn current_branch(root: &Path) -> Result<String> {
    if let Ok(b) = std::env::var("BITBUCKET_BRANCH") {
        if !b.is_empty() {
            return Ok(b);
        }
    }
    git(root, &["symbolic-ref", "--short", "HEAD"])
        .or_else(|_| git(root, &["rev-parse", "--abbrev-ref", "HEAD"]))
        .map(|s| s.trim().to_string())
}

/// Base branch name: explicit → `BITBUCKET_PR_DESTINATION_BRANCH` → `ci.base`.
pub fn base_name(cfg: &Config, explicit: Option<&str>) -> String {
    explicit
        .map(String::from)
        .or_else(|| {
            std::env::var("BITBUCKET_PR_DESTINATION_BRANCH")
                .ok()
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| cfg.ci.base.clone())
}

/// Merge-base commit of HEAD and the base branch (`origin/<name>` first), if reachable.
pub fn resolve_base(root: &Path, cfg: &Config, explicit: Option<&str>) -> Option<String> {
    let name = base_name(cfg, explicit);
    [format!("origin/{name}"), name].iter().find_map(|r| {
        git(
            root,
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{r}^{{commit}}"),
            ],
        )
        .ok()?;
        git(root, &["merge-base", r, "HEAD"])
            .ok()
            .map(|s| s.trim().to_string())
    })
}

fn names(out: &str) -> Vec<String> {
    out.split('\0')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

pub fn changed_files(root: &Path, base: &str) -> Result<Vec<String>> {
    Ok(names(&git(
        root,
        &["diff", "--name-only", "-z", base, "HEAD"],
    )?))
}

pub fn added_files(root: &Path, base: &str) -> Result<Vec<String>> {
    Ok(names(&git(
        root,
        &["diff", "--name-only", "-z", "--diff-filter=A", base, "HEAD"],
    )?))
}

/// Added + deleted lines between `base` and HEAD; binary files count 0.
pub fn diff_lines(root: &Path, base: &str, exclude: &BTreeSet<String>) -> Result<u32> {
    let mut total = 0u32;
    for line in git(root, &["diff", "--numstat", base, "HEAD"])?.lines() {
        let mut parts = line.splitn(3, '\t');
        let (a, d, path) = (parts.next(), parts.next(), parts.next().unwrap_or(""));
        if exclude.contains(path) {
            continue;
        }
        total += a.and_then(|x| x.parse::<u32>().ok()).unwrap_or(0);
        total += d.and_then(|x| x.parse::<u32>().ok()).unwrap_or(0);
    }
    Ok(total)
}

pub fn authors(root: &Path, base: &str) -> Result<BTreeSet<String>> {
    Ok(git(
        root,
        &[
            "log",
            "--no-merges",
            "--format=%an",
            &format!("{base}..HEAD"),
        ],
    )?
    .lines()
    .map(String::from)
    .collect())
}

pub fn user_name(root: &Path) -> Result<String> {
    Ok(git(root, &["config", "user.name"])?.trim().to_string())
}

/// `git config <key> <value>` in this repo.
pub fn set_config(root: &Path, key: &str, value: &str) -> Result<()> {
    git(root, &["config", key, value]).map(|_| ())
}

pub fn get_config(root: &Path, key: &str) -> Option<String> {
    git(root, &["config", "--get", key])
        .ok()
        .map(|s| s.trim().to_string())
}

pub fn switch_new(root: &Path, branch: &str) -> Result<()> {
    git(root, &["switch", "-c", branch]).map(|_| ())
}

/// `git check-ignore`: true when `rel` is ignored (false outside a repo).
pub fn is_ignored(root: &Path, rel: &str) -> bool {
    git(root, &["check-ignore", "-q", "--", rel]).is_ok()
}

/// Paths under `paths` with unstaged edits or untracked (non-ignored) files.
pub fn unstaged(root: &Path, paths: &[&str]) -> Result<Vec<String>> {
    let mut args = vec!["diff", "--name-only", "-z", "--"];
    args.extend(paths);
    let mut out = names(&git(root, &args)?);
    let mut args = vec!["ls-files", "--others", "--exclude-standard", "-z", "--"];
    args.extend(paths);
    out.extend(names(&git(root, &args)?));
    out.sort();
    out.dedup();
    Ok(out)
}
