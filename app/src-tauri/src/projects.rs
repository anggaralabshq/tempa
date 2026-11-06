//! Projects the user added: `projects.json` in the app config dir, written atomically.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    /// Stable id derived from the canonical path.
    pub id: String,
    pub name: String,
    pub path: String,
}

fn load(file: &Path) -> Vec<Project> {
    let Ok(text) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    serde_json::from_str(&text).unwrap_or_else(|_| {
        // Unreadable list: keep it for the user, start a new one.
        let _ = std::fs::rename(file, file.with_extension("json.bak"));
        Vec::new()
    })
}

fn save(file: &Path, list: &[Project]) -> Result<(), String> {
    let text = serde_json::to_string_pretty(list).map_err(|e| e.to_string())?;
    tempa_core::write_atomic(file, text.as_bytes()).map_err(|e| e.0)
}

/// Windows `canonicalize` returns `\\?\C:\...`; shells and people want `C:\...`.
fn display_path(p: &str) -> String {
    match p.strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => rest.to_string(),
        _ => p.to_string(),
    }
}

pub fn list(file: &Path) -> Vec<Project> {
    load(file)
}

/// Add a folder (same folder twice returns the existing entry).
/// Stable project id from its display path (what the frontend holds as `path`).
pub fn id(path: &str) -> String {
    tempa_core::sync::hash(path.as_bytes())[..12].to_string()
}

/// Host data for one project: `<data>/projects/<id>/`.
pub fn data_dir(data: &Path, path: &str) -> std::path::PathBuf {
    data.join("projects").join(id(path))
}

pub fn add(file: &Path, folder: &Path) -> Result<Project, String> {
    let canon = std::fs::canonicalize(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    if !canon.is_dir() {
        return Err(format!("{} is not a folder", canon.display()));
    }
    let path = display_path(&canon.to_string_lossy());
    let mut all = load(file);
    if let Some(p) = all.iter().find(|p| p.path == path) {
        return Ok(p.clone());
    }
    let p = Project {
        id: id(&path),
        name: canon
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone()),
        path,
    };
    all.push(p.clone());
    save(file, &all)?;
    Ok(p)
}

pub fn remove(file: &Path, id: &str) -> Result<(), String> {
    let mut all = load(file);
    all.retain(|p| p.id != id);
    save(file, &all)
}

#[cfg(test)]
mod tests {
    #[test]
    fn data_dir_is_keyed_by_project_id() {
        let d = super::data_dir(std::path::Path::new("/data"), "/src/api");
        assert_eq!(
            d,
            std::path::Path::new("/data/projects").join(super::id("/src/api"))
        );
        assert_eq!(super::id("/src/api").len(), 12);
    }

    use super::*;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let repo = d.path().join("my-repo");
        std::fs::create_dir(&repo).unwrap();
        let file = d.path().join("cfg/projects.json");
        (d, repo, file)
    }

    #[test]
    fn add_is_idempotent_per_path() {
        let (_d, repo, file) = setup();
        let a = add(&file, &repo).unwrap();
        let b = add(&file, &repo.join(".")).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.name, "my-repo");
        assert_eq!(list(&file).len(), 1);
    }

    #[test]
    fn missing_folder_is_an_error() {
        let (_d, repo, file) = setup();
        assert!(add(&file, &repo.join("nope")).is_err());
        assert!(list(&file).is_empty());
    }

    #[test]
    fn verbatim_drive_prefix_is_dropped() {
        assert_eq!(display_path(r"\\?\C:\src\repo"), r"C:\src\repo");
        assert_eq!(display_path(r"\\?\UNC\host\share"), r"\\?\UNC\host\share");
        assert_eq!(display_path("/Users/me/repo"), "/Users/me/repo");
    }

    #[test]
    fn remove_persists() {
        let (_d, repo, file) = setup();
        let p = add(&file, &repo).unwrap();
        remove(&file, &p.id).unwrap();
        assert!(list(&file).is_empty());
    }

    #[test]
    fn corrupt_file_starts_empty_and_is_kept_as_backup() {
        let (_d, repo, file) = setup();
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "{not json").unwrap();
        assert!(list(&file).is_empty());
        add(&file, &repo).unwrap();
        let backup = file.with_extension("json.bak");
        assert_eq!(std::fs::read_to_string(backup).unwrap(), "{not json");
        assert_eq!(list(&file).len(), 1);
    }
}
