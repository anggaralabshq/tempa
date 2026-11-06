//! `tempa init`: create missing contract files, hooks and the DevOps handoff doc.

use crate::config::{self, Config};
use crate::{Result, gitx, write_atomic};
use std::path::Path;

const PRE_COMMIT: &str = include_str!("templates/pre-commit");
const PRE_PUSH: &str = include_str!("templates/pre-push");
const DEVOPS: &str = include_str!("templates/devops.md");
pub const HOOKS_DIR: &str = ".tempa/hooks";

fn heading_doc(title: Option<&str>, headings: &[String]) -> String {
    let mut s = title.map(|t| format!("# {t}\n\n")).unwrap_or_default();
    for h in headings {
        s += &format!("## {h}\n\n");
    }
    s
}

fn project_name(root: &Path) -> String {
    std::fs::canonicalize(root)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".into())
}

/// Create files that do not exist yet (never overwrites). Returns created paths.
pub fn init(root: &Path) -> Result<Vec<String>> {
    let mut created = Vec::new();
    let mut create = |rel: &str, text: &str| -> Result<()> {
        let p = root.join(rel);
        if !p.exists() {
            write_atomic(&p, text.as_bytes())?;
            created.push(rel.to_string());
        }
        Ok(())
    };

    create(config::FILE, &config::default_toml(&project_name(root)))?;
    let cfg = Config::load(root)?;
    let d = &cfg.docs;
    create(
        &d.context,
        &heading_doc(Some(&cfg.project.name), &cfg.headings.context),
    )?;
    create(
        &d.harness,
        &heading_doc(Some("Harness"), &cfg.headings.harness),
    )?;
    create(&d.changes, "# Changes\n")?;
    create(&format!("{}/.gitkeep", d.plan.trim_end_matches('/')), "")?;
    create(
        &format!("{}/.gitkeep", d.decisions.trim_end_matches('/')),
        "",
    )?;
    create(&format!("{HOOKS_DIR}/pre-commit"), PRE_COMMIT)?;
    create(&format!("{HOOKS_DIR}/pre-push"), PRE_PUSH)?;
    create(
        "docs/tempa-devops.md",
        &DEVOPS.replace("{version}", env!("CARGO_PKG_VERSION")),
    )?;

    let attr_line = format!("{} merge=union", d.changes);
    let attrs = root.join(".gitattributes");
    let existing = std::fs::read_to_string(&attrs).unwrap_or_default();
    if !existing.lines().any(|l| l.trim() == attr_line) {
        let mut text = existing.clone();
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text += &attr_line;
        text.push('\n');
        write_atomic(&attrs, text.as_bytes())?;
        if existing.is_empty() {
            created.push(".gitattributes".into());
        }
    }

    #[cfg(unix)]
    for hook in ["pre-commit", "pre-push"] {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            root.join(HOOKS_DIR).join(hook),
            std::fs::Permissions::from_mode(0o755),
        )?;
    }

    // Never replace another hook manager (husky, lefthook): the CLI warns instead.
    if gitx::is_repo(root) && gitx::get_config(root, "core.hooksPath").is_none() {
        gitx::set_config(root, "core.hooksPath", HOOKS_DIR)?;
    }
    Ok(created)
}

/// Agent context that `tempa import` should absorb instead of `tempa init`
/// overwriting it with templates and generated targets.
pub fn existing_agent_context(root: &Path) -> Option<&'static str> {
    if root.join(config::FILE).exists() {
        return None;
    }
    [
        "CLAUDE.md",
        ".claude",
        "GEMINI.md",
        ".cursorrules",
        ".cursor/rules",
        ".github/copilot-instructions.md",
    ]
    .into_iter()
    .find(|p| root.join(p).exists())
}
