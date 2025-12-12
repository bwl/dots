//! Ideas domain types and loader.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

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
