//! Projects domain types and inventory loader.

use anyhow::Result;
use serde::Deserialize;

use crate::paths::IdeasPaths;

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub source: String,
    pub category: String,
    pub tech: String,
    pub last_commit: String,
    pub commits: u32,
    #[serde(default)]
    pub summary: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct ProjectInventory {
    projects: Vec<Project>,
}

/// Load projects from the inventory JSON using provided paths.
/// Returns an empty vector if the inventory file doesn't exist.
pub fn load_projects_with_paths(paths: &IdeasPaths) -> Result<Vec<Project>> {
    let inventory_path = &paths.project_inventory_path;

    if !inventory_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(inventory_path)?;
    let inventory: ProjectInventory = serde_json::from_str(&content)?;

    Ok(inventory.projects)
}

/// Load projects from the inventory JSON.
/// Returns an empty vector if the inventory file doesn't exist.
pub fn load_projects() -> Result<Vec<Project>> {
    let paths = IdeasPaths::detect()?;
    load_projects_with_paths(&paths)
}
