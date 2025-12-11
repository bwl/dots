//! Shared data types and loading functions for the ideas ecosystem.
//!
//! This crate provides common functionality used by both `icli` and `ideas-tui`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

// ============ Ideas ============

#[derive(Debug, Clone)]
pub struct Idea {
    pub folder: String,
    pub tags: Vec<String>,
    pub description: String,
    pub created: String,
    pub modified: String,
    pub sessions: u32,
    pub status: String,
    pub open_questions: Vec<String>,
}

/// Load ideas from the tracker CSV in the given repo root.
pub fn load_ideas(repo_root: &Path) -> Result<Vec<Idea>> {
    let tracker_path = repo_root.join("_tracker.csv");
    let mut reader = csv::Reader::from_path(&tracker_path)
        .with_context(|| format!("Failed to open {}", tracker_path.display()))?;

    let mut ideas = Vec::new();

    for result in reader.records() {
        let record = result?;
        let folder = record.get(0).unwrap_or("").to_string();

        if folder.is_empty() {
            continue;
        }

        let tags = parse_tags(record.get(1).unwrap_or(""));
        let description = record.get(2).unwrap_or("").trim_matches('"').to_string();
        let created = record.get(3).unwrap_or("").to_string();
        let modified = record.get(4).unwrap_or("").to_string();
        let sessions: u32 = record.get(5).unwrap_or("0").parse().unwrap_or(0);

        // Get status and questions from README via mq
        let readme_path = repo_root.join(&folder).join("README.md");
        let status = get_status(&readme_path).unwrap_or_else(|_| "unknown".to_string());
        let open_questions = get_open_questions(&readme_path).unwrap_or_default();

        ideas.push(Idea {
            folder,
            tags,
            description,
            created,
            modified,
            sessions,
            status,
            open_questions,
        });
    }

    Ok(ideas)
}

fn parse_tags(s: &str) -> Vec<String> {
    let s = s.trim_matches('"');
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',').map(|t| t.trim().to_string()).collect()
}

fn get_status(readme_path: &Path) -> Result<String> {
    if !readme_path.exists() {
        return Ok("unknown".to_string());
    }

    let output = Command::new("mq")
        .arg(".")
        .arg(readme_path)
        .output()
        .context("Failed to run mq")?;

    let content = String::from_utf8_lossy(&output.stdout);

    for line in content.lines() {
        if line.contains("Status:") {
            let status = line
                .split("Status:")
                .nth(1)
                .unwrap_or("")
                .trim()
                .trim_matches('*')
                .trim();
            return Ok(status.to_string());
        }
    }

    Ok("unknown".to_string())
}

fn get_open_questions(readme_path: &Path) -> Result<Vec<String>> {
    if !readme_path.exists() {
        return Ok(Vec::new());
    }

    let output = Command::new("mq")
        .arg(".list")
        .arg(readme_path)
        .output()
        .context("Failed to run mq")?;

    let content = String::from_utf8_lossy(&output.stdout);
    let mut questions = Vec::new();

    for line in content.lines() {
        if line.contains("[ ]") {
            let q = line
                .trim_start_matches('-')
                .trim_start_matches('*')
                .trim()
                .to_string();
            questions.push(q);
        }
    }

    Ok(questions)
}

// ============ Projects ============

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub source: String,
    pub category: String,
    pub tech: String,
    pub last_commit: String,
    pub commits: u32,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct ProjectInventory {
    projects: Vec<Project>,
}

/// Load projects from the inventory JSON.
/// Returns an empty vector if the inventory file doesn't exist.
pub fn load_projects() -> Result<Vec<Project>> {
    let home = dirs::home_dir().context("No home directory")?;
    let inventory_path = home.join("Developer/ideas/_data/project-inventory.json");

    if !inventory_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&inventory_path)?;
    let inventory: ProjectInventory = serde_json::from_str(&content)?;

    Ok(inventory.projects)
}

// ============ Plans ============

#[derive(Debug, Clone)]
pub struct Plan {
    pub name: String,
    pub title: String,
    pub modified: String,
    pub path: PathBuf,
}

/// Load Claude plans from ~/.claude/plans.
pub fn load_plans() -> Result<Vec<Plan>> {
    let home = dirs::home_dir().context("No home directory")?;
    let plans_dir = home.join(".claude/plans");

    if !plans_dir.exists() {
        return Ok(Vec::new());
    }

    let mut plans = Vec::new();

    for entry in std::fs::read_dir(&plans_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "md") {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            let modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| {
                    let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                    let secs = duration.as_secs() as i64;
                    Some(chrono_lite(secs))
                })
                .unwrap_or_else(|| "-".to_string());

            let title = get_plan_title(&path).unwrap_or_else(|| "(no title)".to_string());

            plans.push(Plan {
                name,
                title,
                modified,
                path,
            });
        }
    }

    plans.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(plans)
}

fn get_plan_title(path: &Path) -> Option<String> {
    let output = Command::new("mq").arg(".h1").arg(path).output().ok()?;

    let content = String::from_utf8_lossy(&output.stdout);
    let title = content.lines().next()?.trim().trim_start_matches('#').trim();

    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

// ============ Path Resolution ============

/// Find the ideas repository root.
/// Search order: cwd -> git root -> ~/Developer/ideas
pub fn find_ideas_repo() -> Result<PathBuf> {
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

    // Try ~/Developer/ideas
    let home = dirs::home_dir().context("No home directory")?;
    let ideas_path = home.join("Developer/ideas");
    if ideas_path.join("_tracker.csv").exists() {
        return Ok(ideas_path);
    }

    anyhow::bail!("Could not find ideas repo (no _tracker.csv found)")
}

/// Get the analysis directory path.
pub fn analysis_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("No home directory")?;
    Ok(home.join("Developer/ideas/_data/analysis"))
}

/// Check if an analysis file exists for a project.
pub fn has_analysis_file(project_name: &str) -> bool {
    if let Ok(dir) = analysis_dir() {
        dir.join(format!("{}.md", project_name)).exists()
    } else {
        false
    }
}

/// Get analysis file path for a project.
pub fn analysis_file_path(project_name: &str) -> Result<PathBuf> {
    let dir = analysis_dir()?;
    Ok(dir.join(format!("{}.md", project_name)))
}

/// Load the summary section from an analysis file.
pub fn load_analysis_summary(project_name: &str) -> Option<String> {
    let dir = analysis_dir().ok()?;
    let path = dir.join(format!("{}.md", project_name));

    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&path).ok()?;

    let mut in_summary = false;
    let mut output = String::new();

    for line in content.lines() {
        if line.starts_with("# ") || line.starts_with("> ") {
            in_summary = true;
        }
        if line == "---" || line.starts_with("## Deep Dive") {
            break;
        }
        if in_summary {
            output.push_str(line);
            output.push('\n');
        }
    }

    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

// ============ Git Operations ============

/// Get current HEAD commit (short hash) for a project.
pub fn get_project_head_commit(project_path: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", project_path, "rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Count commits since a given commit.
pub fn count_commits_since(project_path: &str, since_commit: &str) -> Option<u32> {
    let output = Command::new("git")
        .args([
            "-C",
            project_path,
            "rev-list",
            "--count",
            &format!("{}..HEAD", since_commit),
        ])
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .ok()
    } else {
        None
    }
}

// ============ Date Utilities ============

/// Convert unix timestamp to YYYY-MM-DD string.
pub fn chrono_lite(unix_secs: i64) -> String {
    let days_since_epoch = unix_secs / 86400;
    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days in days_in_months.iter() {
        if remaining_days < *days {
            break;
        }
        remaining_days -= days;
        month += 1;
    }

    let day = remaining_days + 1;

    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// ============ Dotfiles ============

#[derive(Debug, Clone, Deserialize)]
pub struct DxItem {
    pub name: String,
    pub category: String,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct DxInventory {
    items: Vec<DxItem>,
}

/// Load dotfiles/DX tools inventory.
pub fn load_dotfiles() -> Result<Vec<DxItem>> {
    let home = dirs::home_dir().context("No home directory")?;
    let inventory_path = home.join("dotfiles/_data/dx-inventory.json");

    if !inventory_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&inventory_path)?;
    let inventory: DxInventory = serde_json::from_str(&content)?;

    Ok(inventory.items)
}

// ============ Analysis Metadata ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysisMeta {
    pub analyzed_at: String,
    pub analyzed_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMeta {
    pub version: u32,
    pub projects: HashMap<String, ProjectAnalysisMeta>,
}

impl Default for AnalysisMeta {
    fn default() -> Self {
        Self {
            version: 1,
            projects: HashMap::new(),
        }
    }
}

/// Load analysis metadata from _meta.json.
pub fn load_analysis_meta() -> Result<AnalysisMeta> {
    let home = dirs::home_dir().context("No home directory")?;
    let meta_path = home.join("Developer/ideas/_data/analysis/_meta.json");

    if !meta_path.exists() {
        return Ok(AnalysisMeta::default());
    }

    let content = std::fs::read_to_string(&meta_path)?;
    let meta: AnalysisMeta = serde_json::from_str(&content)?;

    Ok(meta)
}

/// Save analysis metadata to _meta.json.
pub fn save_analysis_meta(meta: &AnalysisMeta) -> Result<()> {
    let home = dirs::home_dir().context("No home directory")?;
    let meta_path = home.join("Developer/ideas/_data/analysis/_meta.json");

    let content = serde_json::to_string_pretty(meta)?;
    std::fs::write(&meta_path, content)?;

    Ok(())
}

// ============ Dirty Detection ============

#[derive(Debug, Clone)]
pub struct DirtyProject {
    pub name: String,
    pub path: String,
    pub analyzed_at: Option<String>,
    pub analyzed_commit: Option<String>,
    pub current_commit: Option<String>,
    pub commits_since: Option<u32>,
}

/// Check if a project is dirty (needs re-analysis).
pub fn check_project_dirty(project: &Project, meta: &AnalysisMeta) -> DirtyProject {
    let analysis_meta = meta.projects.get(&project.name);
    let current_commit = get_project_head_commit(&project.path);

    let (analyzed_at, analyzed_commit, commits_since) = match analysis_meta {
        Some(m) => {
            let since = current_commit
                .as_ref()
                .and_then(|_| count_commits_since(&project.path, &m.analyzed_commit));
            (
                Some(m.analyzed_at.clone()),
                Some(m.analyzed_commit.clone()),
                since,
            )
        }
        None => (None, None, None),
    };

    DirtyProject {
        name: project.name.clone(),
        path: project.path.clone(),
        analyzed_at,
        analyzed_commit,
        current_commit,
        commits_since,
    }
}

// ============ Project Discovery ============

#[derive(Debug, Clone)]
pub struct UntrackedProject {
    pub name: String,
    pub path: PathBuf,
    pub tech: String,
    pub commits: u32,
}

/// Detect projects in ~/Developer that are not in the inventory.
pub fn detect_untracked_projects() -> Result<Vec<UntrackedProject>> {
    let home = dirs::home_dir().context("No home directory")?;
    let dev_dir = home.join("Developer");

    let projects = load_projects().unwrap_or_default();
    let known_paths: std::collections::HashSet<_> = projects
        .iter()
        .map(|p| p.path.trim_end_matches('/').to_string())
        .collect();

    let mut untracked = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dev_dir) {
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

// ============ Recent Activity ============

#[derive(Debug, Clone)]
pub struct RecentProject {
    pub name: String,
    pub last_commit_date: String,
    pub last_commit_msg: String,
}

/// Get projects with commits in the last N days.
pub fn get_recent_activity(days: u32) -> Result<Vec<RecentProject>> {
    let projects = load_projects()?;
    let mut recent = Vec::new();

    for project in &projects {
        let output = Command::new("git")
            .args([
                "-C",
                &project.path,
                "log",
                "-1",
                &format!("--since={} days ago", days),
                "--format=%ci|%s",
            ])
            .output();

        if let Ok(output) = output {
            let line = String::from_utf8_lossy(&output.stdout);
            let line = line.trim();
            if !line.is_empty() {
                let parts: Vec<&str> = line.splitn(2, '|').collect();
                let date = parts
                    .first()
                    .map(|d| d.split_whitespace().next().unwrap_or(""))
                    .unwrap_or("")
                    .to_string();
                let msg = parts.get(1).unwrap_or(&"").to_string();

                if !date.is_empty() {
                    recent.push(RecentProject {
                        name: project.name.clone(),
                        last_commit_date: date,
                        last_commit_msg: msg,
                    });
                }
            }
        }
    }

    recent.sort_by(|a, b| b.last_commit_date.cmp(&a.last_commit_date));

    Ok(recent)
}
