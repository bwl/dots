use anyhow::Result;
use colored::Colorize;

use crate::data::{load_dotfiles_with_paths, IdeasPaths};
use super::util::truncate;

pub fn cmd_dotfiles(paths: &IdeasPaths, category: Option<String>) -> Result<()> {
    let items = load_dotfiles_with_paths(paths)?;

    let filtered: Vec<_> = items
        .iter()
        .filter(|d| {
            if let Some(ref c) = category {
                if d.category != *c {
                    return false;
                }
            }
            true
        })
        .collect();

    println!(
        "{:<20} {:<14} {:<30} {}",
        "NAME".bold(),
        "CATEGORY".bold(),
        "PATH".bold(),
        "DESCRIPTION".bold()
    );

    for item in filtered {
        let cat_colored = match item.category.as_str() {
            "dx-script" => item.category.green(),
            "dx-tool" => item.category.cyan(),
            "shell-config" => item.category.yellow(),
            "app-config" => item.category.blue(),
            "tool-list" => item.category.magenta(),
            "claude-skill" => item.category.red(),
            _ => item.category.white(),
        };

        println!(
            "{:<20} {:<14} {:<30} {}",
            truncate(&item.name, 19),
            cat_colored,
            truncate(&item.path, 29),
            truncate(&item.description, 40)
        );
    }

    Ok(())
}

