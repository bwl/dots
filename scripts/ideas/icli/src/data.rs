use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
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
}

pub fn load_ideas() -> Result<Vec<Idea>> {
    let repo_root = find_ideas_repo()?;
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

        // Get status from README via mq
        let readme_path = repo_root.join(&folder).join("README.md");
        let status = get_status(&readme_path).unwrap_or_else(|_| "unknown".to_string());

        ideas.push(Idea {
            folder,
            tags,
            description,
            created,
            modified,
            sessions,
            status,
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

fn get_status(readme_path: &std::path::Path) -> Result<String> {
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

fn find_ideas_repo() -> Result<PathBuf> {
    // Try ~/Developer/ideas first
    let home = dirs::home_dir().context("No home directory")?;
    let ideas_path = home.join("Developer/ideas");
    if ideas_path.join("_tracker.csv").exists() {
        return Ok(ideas_path);
    }

    // Try current directory
    let cwd = std::env::current_dir()?;
    if cwd.join("_tracker.csv").exists() {
        return Ok(cwd);
    }

    anyhow::bail!("Could not find ideas repo (no _tracker.csv found)")
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
        anyhow::bail!(
            "Project inventory not found. Run 'icli refresh' or '_scripts/mq/projects-scan.sh'"
        );
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

            // Get modification time
            let modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| {
                    let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                    let secs = duration.as_secs() as i64;
                    let dt = chrono_lite(secs);
                    Some(dt)
                })
                .unwrap_or_else(|| "-".to_string());

            // Get title from first h1
            let title = get_plan_title(&path).unwrap_or_else(|| "(no title)".to_string());

            plans.push(Plan {
                name,
                title,
                modified,
                path,
            });
        }
    }

    // Sort by modification date descending
    plans.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(plans)
}

fn get_plan_title(path: &std::path::Path) -> Option<String> {
    let output = Command::new("mq").arg(".h1").arg(path).output().ok()?;

    let content = String::from_utf8_lossy(&output.stdout);
    let title = content.lines().next()?.trim().trim_start_matches('#').trim();

    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

// Simple date formatting without chrono dependency
fn chrono_lite(unix_secs: i64) -> String {
    // Convert unix timestamp to YYYY-MM-DD
    // This is a simplified version - good enough for sorting
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
