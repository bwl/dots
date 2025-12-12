//! Claude plan domain types and loader.

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::paths::IdeasPaths;
use crate::util::chrono_lite;

#[derive(Debug, Clone)]
pub struct Plan {
    pub name: String,
    pub title: String,
    pub modified: String,
    pub path: PathBuf,
}

/// Load Claude plans using provided paths.
pub fn load_plans_with_paths(paths: &IdeasPaths) -> Result<Vec<Plan>> {
    let plans_dir = &paths.claude_plans_dir;

    if !plans_dir.exists() {
        return Ok(Vec::new());
    }

    let mut plans = Vec::new();

    for entry in std::fs::read_dir(plans_dir)? {
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

/// Load Claude plans from ~/.claude/plans.
pub fn load_plans() -> Result<Vec<Plan>> {
    let paths = IdeasPaths::detect()?;
    load_plans_with_paths(&paths)
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
