//! Dotfiles/DX inventory types and loader.

use anyhow::Result;
use serde::Deserialize;

use crate::paths::IdeasPaths;

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

/// Load dotfiles/DX tools inventory using provided paths.
pub fn load_dotfiles_with_paths(paths: &IdeasPaths) -> Result<Vec<DxItem>> {
    let inventory_path = &paths.dx_inventory_path;

    if !inventory_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(inventory_path)?;
    let inventory: DxInventory = serde_json::from_str(&content)?;

    Ok(inventory.items)
}

/// Load dotfiles/DX tools inventory.
pub fn load_dotfiles() -> Result<Vec<DxItem>> {
    let paths = IdeasPaths::detect()?;
    load_dotfiles_with_paths(&paths)
}
