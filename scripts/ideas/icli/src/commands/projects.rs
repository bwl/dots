use anyhow::Result;
use colored::Colorize;

use crate::data::{
    has_analysis_file_with_paths, load_projects_with_paths, normalize_query, project_matches_query, IdeasPaths,
    Project,
};
use super::util::{clean_desc, truncate};

pub fn cmd_projects(
    paths: &IdeasPaths,
    category: Option<String>,
    search: Option<String>,
    group: bool,
    analyzed_only: bool,
    show_analysis: bool,
) -> Result<()> {
    let projects = load_projects_with_paths(paths)?;
    let q_lower = search.as_deref().map(normalize_query);

    let filtered: Vec<_> = projects
        .iter()
        .filter(|p| {
            if analyzed_only && !has_analysis_file_with_paths(paths, &p.name) {
                return false;
            }
            if let Some(ref c) = category {
                if p.category != *c {
                    return false;
                }
            }
            if let Some(ref q) = q_lower {
                if !project_matches_query(p, q) {
                    return false;
                }
            }
            true
        })
        .collect();

    if group {
        let mut categories: std::collections::HashMap<&str, Vec<&Project>> =
            std::collections::HashMap::new();
        for p in &filtered {
            categories.entry(&p.category).or_default().push(p);
        }

        let mut cats: Vec<_> = categories.keys().collect();
        cats.sort();

        for cat in cats {
            let projects = &categories[cat];
            println!("\n{} ({})", format!("=== {} ===", cat).cyan(), projects.len());
            for p in projects {
                let analyzed = if has_analysis_file_with_paths(paths, &p.name) {
                    "[A]".green()
                } else {
                    "[ ]".dimmed()
                };
                let desc = clean_desc(if !p.summary.is_empty() { &p.summary } else { &p.description });
                if show_analysis {
                    println!(
                        "  {} {:<18} {:<12} {}",
                        analyzed,
                        p.name,
                        &p.last_commit,
                        truncate(&desc, 30)
                    );
                } else {
                    println!(
                        "  {:<20} {:<12} {}",
                        p.name,
                        &p.last_commit,
                        truncate(&desc, 35)
                    );
                }
            }
        }
    } else {
        if show_analysis {
            println!(
                "{:<4} {:<20} {:<12} {:<12} {}",
                "".bold(),
                "NAME".bold(),
                "CATEGORY".bold(),
                "LAST_COMMIT".bold(),
                "DESCRIPTION".bold()
            );
        } else {
            println!(
                "{:<22} {:<12} {:<12} {:<10} {}",
                "NAME".bold(),
                "CATEGORY".bold(),
                "LAST_COMMIT".bold(),
                "SOURCE".bold(),
                "DESCRIPTION".bold()
            );
        }

        for p in filtered {
            let cat_colored = match p.category.as_str() {
                "roguelike" => p.category.red(),
                "writing" => p.category.magenta(),
                "knowledge" => p.category.blue(),
                "simulation" => p.category.green(),
                "tui" | "cli" => p.category.cyan(),
                _ => p.category.white(),
            };

            let desc = clean_desc(if !p.summary.is_empty() { &p.summary } else { &p.description });
            if show_analysis {
                let analyzed = if has_analysis_file_with_paths(paths, &p.name) {
                    "[A]".green()
                } else {
                    "[ ]".dimmed()
                };
                println!(
                    "{} {:<20} {:<12} {:<12} {}",
                    analyzed,
                    truncate(&p.name, 19),
                    cat_colored,
                    &p.last_commit,
                    truncate(&desc, 30)
                );
            } else {
                println!(
                    "{:<22} {:<12} {:<12} {:<10} {}",
                    truncate(&p.name, 21),
                    cat_colored,
                    &p.last_commit,
                    &p.source,
                    truncate(&desc, 30)
                );
            }
        }
    }

    Ok(())
}

