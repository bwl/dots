use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
#[allow(dead_code)]
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


pub fn load_ideas(repo_root: &Path) -> Result<Vec<Idea>> {
    let tracker_path = repo_root.join("_tracker.csv");
    let mut reader = csv::Reader::from_path(&tracker_path)
        .with_context(|| format!("Failed to open {}", tracker_path.display()))?;

    let mut ideas = Vec::new();

    for result in reader.records() {
        let record = result?;
        let folder = record.get(0).unwrap_or("").to_string();

        // Skip empty rows
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

    // Look for "Status:" line
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
            // Clean up the question text
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Name,
    Status,
    Questions,
    Sessions,
    Modified,
}

impl SortBy {
    pub fn next(self) -> Self {
        match self {
            SortBy::Name => SortBy::Status,
            SortBy::Status => SortBy::Questions,
            SortBy::Questions => SortBy::Sessions,
            SortBy::Sessions => SortBy::Modified,
            SortBy::Modified => SortBy::Name,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortBy::Name => "name",
            SortBy::Status => "status",
            SortBy::Questions => "questions",
            SortBy::Sessions => "sessions",
            SortBy::Modified => "modified",
        }
    }
}

pub fn sort_ideas(ideas: &mut [Idea], sort_by: SortBy) {
    match sort_by {
        SortBy::Name => ideas.sort_by(|a, b| a.folder.cmp(&b.folder)),
        SortBy::Status => ideas.sort_by(|a, b| a.status.cmp(&b.status)),
        SortBy::Questions => {
            ideas.sort_by(|a, b| b.open_questions.len().cmp(&a.open_questions.len()))
        }
        SortBy::Sessions => ideas.sort_by(|a, b| b.sessions.cmp(&a.sessions)),
        SortBy::Modified => ideas.sort_by(|a, b| b.modified.cmp(&a.modified)),
    }
}

/// Find all markdown files in an idea folder, with README.md first
pub fn find_markdown_files(idea_path: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(idea_path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "md"))
        .collect();

    // Sort with README.md first, then alphabetically
    files.sort_by(|a, b| {
        let a_readme = a.file_name().map_or(false, |n| n == "README.md");
        let b_readme = b.file_name().map_or(false, |n| n == "README.md");
        match (a_readme, b_readme) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        }
    });
    files
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

pub fn analysis_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("No home directory")?;
    Ok(home.join("Developer/ideas/_data/analysis"))
}

pub fn has_analysis_file(project_name: &str) -> bool {
    if let Ok(dir) = analysis_dir() {
        dir.join(format!("{}.md", project_name)).exists()
    } else {
        false
    }
}

pub fn load_analysis_summary(project_name: &str) -> Option<String> {
    let dir = analysis_dir().ok()?;
    let path = dir.join(format!("{}.md", project_name));

    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&path).ok()?;

    // Extract summary section (up to "---" or "## Deep Dive")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectSortBy {
    Name,
    Category,
    LastCommit,
    Analyzed,
}

impl ProjectSortBy {
    pub fn next(self) -> Self {
        match self {
            ProjectSortBy::Name => ProjectSortBy::Category,
            ProjectSortBy::Category => ProjectSortBy::LastCommit,
            ProjectSortBy::LastCommit => ProjectSortBy::Analyzed,
            ProjectSortBy::Analyzed => ProjectSortBy::Name,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ProjectSortBy::Name => "name",
            ProjectSortBy::Category => "category",
            ProjectSortBy::LastCommit => "last commit",
            ProjectSortBy::Analyzed => "analyzed",
        }
    }
}

pub fn sort_projects(projects: &mut [Project], sort_by: ProjectSortBy) {
    match sort_by {
        ProjectSortBy::Name => projects.sort_by(|a, b| a.name.cmp(&b.name)),
        ProjectSortBy::Category => projects.sort_by(|a, b| a.category.cmp(&b.category)),
        ProjectSortBy::LastCommit => projects.sort_by(|a, b| b.last_commit.cmp(&a.last_commit)),
        ProjectSortBy::Analyzed => {
            projects.sort_by(|a, b| {
                let a_has = has_analysis_file(&a.name);
                let b_has = has_analysis_file(&b.name);
                b_has.cmp(&a_has)
            })
        }
    }
}
