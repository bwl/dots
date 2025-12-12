use anyhow::Result;
use colored::Colorize;

use crate::data::{find_ideas_repo_with_paths, idea_matches_query, load_ideas, normalize_query, IdeasPaths};
use super::util::truncate;

pub fn cmd_ideas(paths: &IdeasPaths, status: Option<String>, search: Option<String>) -> Result<()> {
    let repo_root = find_ideas_repo_with_paths(paths)?;
    let ideas = load_ideas(&repo_root)?;
    let q_lower = search.as_deref().map(normalize_query);

    let filtered: Vec<_> = ideas
        .iter()
        .filter(|i| {
            if let Some(ref s) = status {
                if i.status != *s {
                    return false;
                }
            }
            if let Some(ref q) = q_lower {
                if !idea_matches_query(i, q) {
                    return false;
                }
            }
            true
        })
        .collect();

    println!(
        "{:<22} {:<10} {:<12} {}",
        "FOLDER".bold(),
        "STATUS".bold(),
        "MODIFIED".bold(),
        "DESCRIPTION".bold()
    );

    for idea in filtered {
        let status_colored = match idea.status.as_str() {
            "active" => idea.status.green(),
            "dormant" => idea.status.yellow(),
            _ => idea.status.red(),
        };

        println!(
            "{:<22} {:<10} {:<12} {}",
            truncate(&idea.folder, 21),
            status_colored,
            &idea.modified,
            truncate(&idea.description, 40)
        );
    }

    Ok(())
}

