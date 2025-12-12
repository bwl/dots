//! Recent activity helpers.

use anyhow::Result;
use std::process::Command;

use crate::paths::IdeasPaths;
use crate::projects::load_projects_with_paths;

#[derive(Debug, Clone)]
pub struct RecentProject {
    pub name: String,
    pub last_commit_date: String,
    pub last_commit_msg: String,
}

/// Get projects with commits in the last N days.
pub fn get_recent_activity_with_paths(
    paths: &IdeasPaths,
    days: u32,
) -> Result<Vec<RecentProject>> {
    let projects = load_projects_with_paths(paths)?;
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

/// Get projects with commits in the last N days.
pub fn get_recent_activity(days: u32) -> Result<Vec<RecentProject>> {
    let paths = IdeasPaths::detect()?;
    get_recent_activity_with_paths(&paths, days)
}
