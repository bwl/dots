//! Filesystem path resolution for ideas data.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Resolved filesystem locations for ideas-related data.
///
/// Defaults match the original hard-coded layout, but can be overridden via env vars:
/// - `IDEAS_REPO` (default: `~/Developer/ideas`)
/// - `IDEAS_DEVELOPER_DIR` (default: `~/Developer`)
/// - `DOTFILES_REPO` (default: `~/dotfiles`)
/// - `IDEAS_CLAUDE_PLANS_DIR` (default: `~/.claude/plans`)
#[derive(Debug, Clone)]
pub struct IdeasPaths {
    pub ideas_repo: PathBuf,
    pub ideas_data_dir: PathBuf,
    pub analysis_dir: PathBuf,
    pub project_inventory_path: PathBuf,
    pub analysis_meta_path: PathBuf,
    pub developer_dir: PathBuf,
    pub dotfiles_repo: PathBuf,
    pub dx_inventory_path: PathBuf,
    pub claude_plans_dir: PathBuf,
}

impl IdeasPaths {
    /// Detect paths from the environment and home directory.
    pub fn detect() -> Result<Self> {
        let home = dirs::home_dir().context("No home directory")?;

        let ideas_repo = env_path("IDEAS_REPO").unwrap_or_else(|| home.join("Developer/ideas"));
        let ideas_data_dir = ideas_repo.join("_data");
        let analysis_dir = ideas_data_dir.join("analysis");
        let project_inventory_path = ideas_data_dir.join("project-inventory.json");
        let analysis_meta_path = analysis_dir.join("_meta.json");

        let developer_dir =
            env_path("IDEAS_DEVELOPER_DIR").unwrap_or_else(|| home.join("Developer"));

        let dotfiles_repo = env_path("DOTFILES_REPO").unwrap_or_else(|| home.join("dotfiles"));
        let dx_inventory_path = dotfiles_repo.join("_data").join("dx-inventory.json");

        let claude_plans_dir =
            env_path("IDEAS_CLAUDE_PLANS_DIR").unwrap_or_else(|| home.join(".claude/plans"));

        Ok(Self {
            ideas_repo,
            ideas_data_dir,
            analysis_dir,
            project_inventory_path,
            analysis_meta_path,
            developer_dir,
            dotfiles_repo,
            dx_inventory_path,
            claude_plans_dir,
        })
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).map(PathBuf::from)
}

/// Find the ideas repository root using a provided paths config.
/// Search order: cwd -> git root -> paths.ideas_repo
pub fn find_ideas_repo_with_paths(paths: &IdeasPaths) -> Result<PathBuf> {
    // Try current directory first
    let cwd = std::env::current_dir()?;
    if cwd.join("_tracker.csv").exists() {
        return Ok(cwd);
    }

    // Try git root of current directory
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let git_root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
            if git_root.join("_tracker.csv").exists() {
                return Ok(git_root);
            }
        }
    }

    if paths.ideas_repo.join("_tracker.csv").exists() {
        return Ok(paths.ideas_repo.clone());
    }

    anyhow::bail!("Could not find ideas repo (no _tracker.csv found)")
}

/// Find the ideas repository root.
/// Search order: cwd -> git root -> ~/Developer/ideas (or `IDEAS_REPO`)
pub fn find_ideas_repo() -> Result<PathBuf> {
    let paths = IdeasPaths::detect()?;
    find_ideas_repo_with_paths(&paths)
}

/// Get the analysis directory path for a provided paths config.
pub fn analysis_dir_with_paths(paths: &IdeasPaths) -> PathBuf {
    paths.analysis_dir.clone()
}

/// Get the analysis directory path.
pub fn analysis_dir() -> Result<PathBuf> {
    Ok(IdeasPaths::detect()?.analysis_dir)
}

/// Check if an analysis file exists for a project using provided paths.
pub fn has_analysis_file_with_paths(paths: &IdeasPaths, project_name: &str) -> bool {
    paths
        .analysis_dir
        .join(format!("{}.md", project_name))
        .exists()
}

/// Check if an analysis file exists for a project.
pub fn has_analysis_file(project_name: &str) -> bool {
    IdeasPaths::detect()
        .ok()
        .map(|p| has_analysis_file_with_paths(&p, project_name))
        .unwrap_or(false)
}

/// Get analysis file path for a project using provided paths.
pub fn analysis_file_path_with_paths(paths: &IdeasPaths, project_name: &str) -> PathBuf {
    paths.analysis_dir.join(format!("{}.md", project_name))
}

/// Get analysis file path for a project.
pub fn analysis_file_path(project_name: &str) -> Result<PathBuf> {
    let paths = IdeasPaths::detect()?;
    Ok(analysis_file_path_with_paths(&paths, project_name))
}
