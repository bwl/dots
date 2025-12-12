use anyhow::Result;
use colored::Colorize;

use crate::data::{
    dxitem_matches_query, find_ideas_repo_with_paths, idea_matches_query, load_dotfiles_with_paths,
    load_ideas, load_plans_with_paths, load_projects_with_paths, normalize_query,
    plan_matches_query, project_matches_query, IdeasPaths,
};
use super::util::truncate;

pub fn cmd_search(paths: &IdeasPaths, query: &str) -> Result<()> {
    let q = normalize_query(query);

    let repo_root = find_ideas_repo_with_paths(paths)?;
    let ideas = load_ideas(&repo_root)?;
    let idea_matches: Vec<_> = ideas
        .iter()
        .filter(|i| idea_matches_query(i, &q))
        .collect();

    let projects = load_projects_with_paths(paths)?;
    let project_matches: Vec<_> = projects
        .iter()
        .filter(|p| project_matches_query(p, &q))
        .collect();

    let plans = load_plans_with_paths(paths)?;
    let plan_matches: Vec<_> = plans
        .iter()
        .filter(|p| plan_matches_query(p, &q))
        .collect();

    let dotfiles = load_dotfiles_with_paths(paths)?;
    let dotfile_matches: Vec<_> = dotfiles
        .iter()
        .filter(|d| dxitem_matches_query(d, &q))
        .collect();

    if !idea_matches.is_empty() {
        println!("{}", format!("\n=== Ideas ({}) ===", idea_matches.len()).yellow());
        for i in &idea_matches {
            println!(
                "  {:<20} [{}] {}",
                i.folder,
                i.status,
                truncate(&i.description, 40)
            );
        }
    }

    if !project_matches.is_empty() {
        println!(
            "{}",
            format!("\n=== Projects ({}) ===", project_matches.len()).cyan()
        );
        for p in &project_matches {
            let desc = if !p.summary.is_empty() { &p.summary } else { &p.description };
            println!(
                "  {:<20} [{}] {}",
                p.name,
                p.category,
                truncate(desc, 40)
            );
        }
    }

    if !plan_matches.is_empty() {
        println!(
            "{}",
            format!("\n=== Plans ({}) ===", plan_matches.len()).magenta()
        );
        for p in &plan_matches {
            println!("  {:<30} {}", p.name, truncate(&p.title, 45));
        }
    }

    if !dotfile_matches.is_empty() {
        println!(
            "{}",
            format!("\n=== Dotfiles ({}) ===", dotfile_matches.len()).blue()
        );
        for d in &dotfile_matches {
            println!(
                "  {:<20} [{}] {}",
                d.name,
                d.category,
                truncate(&d.description, 40)
            );
        }
    }

    let total =
        idea_matches.len() + project_matches.len() + plan_matches.len() + dotfile_matches.len();
    println!("\n{} total matches for '{}'", total.to_string().bold(), query);

    Ok(())
}

