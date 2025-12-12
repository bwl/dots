//! Project discovery outside the inventory.

use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

use crate::paths::IdeasPaths;
use crate::projects::load_projects_with_paths;

#[derive(Debug, Clone)]
pub struct UntrackedProject {
    pub name: String,
    pub path: PathBuf,
    pub tech: String,
    pub commits: u32,
}

/// Detect projects in ~/Developer that are not in the inventory.
pub fn detect_untracked_projects_with_paths(paths: &IdeasPaths) -> Result<Vec<UntrackedProject>> {
    let dev_dir = &paths.developer_dir;

    let projects = load_projects_with_paths(paths).unwrap_or_default();
    let known_paths: HashSet<_> = projects
        .iter()
        .map(|p| p.path.trim_end_matches('/').to_string())
        .collect();

    let mut untracked = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dev_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            if known_paths.contains(&path_str) || known_paths.contains(&format!("{}/", path_str)) {
                continue;
            }

            let is_git = path.join(".git").exists();
            let has_cargo = path.join("Cargo.toml").exists();
            let has_package = path.join("package.json").exists();
            let has_go = path.join("go.mod").exists();

            if !is_git && !has_cargo && !has_package && !has_go {
                continue;
            }

            let tech = if has_cargo {
                "rust".to_string()
            } else if has_go {
                "go".to_string()
            } else if has_package {
                "js".to_string()
            } else {
                "unknown".to_string()
            };

            let commits = if is_git {
                Command::new("git")
                    .args(["-C", path.to_str().unwrap_or(""), "rev-list", "--count", "HEAD"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
                    .unwrap_or(0)
            } else {
                0
            };

            untracked.push(UntrackedProject {
                name,
                path,
                tech,
                commits,
            });
        }
    }

    untracked.sort_by(|a, b| b.commits.cmp(&a.commits));

    Ok(untracked)
}

/// Detect projects in ~/Developer that are not in the inventory.
pub fn detect_untracked_projects() -> Result<Vec<UntrackedProject>> {
    let paths = IdeasPaths::detect()?;
    detect_untracked_projects_with_paths(&paths)
}
