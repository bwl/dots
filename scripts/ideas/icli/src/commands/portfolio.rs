use anyhow::Result;
use colored::Colorize;

use crate::data::{
    check_project_dirty, detect_untracked_projects_with_paths, find_ideas_repo_with_paths,
    get_recent_activity_with_paths, has_analysis_file_with_paths, load_analysis_meta_with_paths,
    load_dotfiles_with_paths, load_ideas, load_plans_with_paths, load_projects_with_paths,
    IdeasPaths,
};
use super::util::truncate;

pub fn cmd_stats(paths: &IdeasPaths) -> Result<()> {
    let repo_root = find_ideas_repo_with_paths(paths)?;
    let ideas = load_ideas(&repo_root)?;
    let projects = load_projects_with_paths(paths)?;
    let plans = load_plans_with_paths(paths)?;
    let dotfiles = load_dotfiles_with_paths(paths)?;

    println!("{}", "=== Portfolio Stats ===".bold());
    println!();

    let active = ideas.iter().filter(|i| i.status == "active").count();
    let dormant = ideas.iter().filter(|i| i.status == "dormant").count();
    println!(
        "{}: {} ({} active, {} dormant)",
        "Ideas".yellow(),
        ideas.len(),
        active,
        dormant
    );

    let analysis_meta = load_analysis_meta_with_paths(paths).unwrap_or_default();
    let analyzed_count = analysis_meta.projects.len();
    println!(
        "{}: {} ({} analyzed)",
        "Projects".cyan(),
        projects.len(),
        analyzed_count
    );
    let mut cat_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for p in &projects {
        *cat_counts.entry(&p.category).or_default() += 1;
    }
    let mut cats: Vec<_> = cat_counts.iter().collect();
    cats.sort_by(|a, b| b.1.cmp(a.1));
    for (cat, count) in cats.iter().take(5) {
        println!("  {}: {}", cat, count);
    }

    println!("{}: {}", "Plans".magenta(), plans.len());

    let mut dx_cats: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for d in &dotfiles {
        *dx_cats.entry(&d.category).or_default() += 1;
    }
    let dx_summary: Vec<_> = dx_cats
        .iter()
        .map(|(c, n)| format!("{} {}", n, c))
        .collect();
    println!(
        "{}: {} ({})",
        "Dotfiles".blue(),
        dotfiles.len(),
        dx_summary.join(", ")
    );

    println!();
    println!(
        "{}: {}",
        "Total items".bold(),
        ideas.len() + projects.len() + plans.len() + dotfiles.len()
    );

    Ok(())
}

pub fn cmd_status(paths: &IdeasPaths) -> Result<()> {
    println!("{}", "=== Portfolio Status ===".bold());
    println!();

    let untracked = detect_untracked_projects_with_paths(paths)?;
    if !untracked.is_empty() {
        println!("{}", "⚠ New Projects (not in inventory):".yellow());
        for p in untracked.iter().take(5) {
            println!(
                "  {:<18} {} ({} commits, {})",
                p.name.cyan(),
                p.path.display(),
                p.commits,
                p.tech
            );
        }
        if untracked.len() > 5 {
            println!("  ... {} more", untracked.len() - 5);
        }
        println!("  Run {} to add them", "icli refresh".green());
        println!();
    }

    let projects = load_projects_with_paths(paths)?;
    let meta = load_analysis_meta_with_paths(paths)?;
    let mut needs_analysis: Vec<_> = projects
        .iter()
        .filter(|p| !has_analysis_file_with_paths(paths, &p.name) && p.commits > 0)
        .map(|p| (p.name.clone(), p.commits, p.category.clone()))
        .collect();
    needs_analysis.sort_by(|a, b| b.1.cmp(&a.1));

    if !needs_analysis.is_empty() {
        println!(
            "{}",
            format!("⚠ Needs Analysis ({} projects):", needs_analysis.len()).yellow()
        );
        for (name, commits, category) in needs_analysis.iter().take(5) {
            println!(
                "  {:<18} {} commits  [{}]",
                name.cyan(),
                commits,
                category
            );
        }
        if needs_analysis.len() > 5 {
            println!(
                "  ... {} more (run {})",
                needs_analysis.len() - 5,
                "icli dirty".green()
            );
        }
        println!();
    }

    let mut stale: Vec<_> = projects
        .iter()
        .filter_map(|p| {
            if !has_analysis_file_with_paths(paths, &p.name) {
                return None;
            }
            let dirty = check_project_dirty(p, &meta);
            let commits = dirty.commits_since.unwrap_or(0);
            if commits > 0 {
                Some((p.name.clone(), commits))
            } else {
                None
            }
        })
        .collect();
    stale.sort_by(|a, b| b.1.cmp(&a.1));

    if !stale.is_empty() {
        println!(
            "{}",
            format!("⚠ Stale Analyses ({} total):", stale.len()).yellow()
        );
        for (name, commits) in stale.iter().take(3) {
            println!(
                "  {:<18} {} commits since analysis",
                name.cyan(),
                commits.to_string().red()
            );
        }
        if stale.len() > 3 {
            println!("  ... run {} for full list", "icli dirty".green());
        }
        println!();
    }

    let recent = get_recent_activity_with_paths(paths, 7)?;
    if !recent.is_empty() {
        println!("{}", "Recent Activity (last 7 days):".blue());
        for p in recent.iter().take(5) {
            println!(
                "  {:<18} {}  {}",
                p.name.cyan(),
                p.last_commit_date,
                truncate(&p.last_commit_msg, 40)
            );
        }
        if recent.len() > 5 {
            println!("  ... {} more active projects", recent.len() - 5);
        }
        println!();
    }

    let ideas_repo = find_ideas_repo_with_paths(paths)?;
    let ideas = load_ideas(&ideas_repo)?;
    let analyzed_count = meta.projects.len();
    println!(
        "{}: {} projects │ {} analyzed │ {} ideas",
        "Quick Stats".bold(),
        projects.len(),
        analyzed_count,
        ideas.len()
    );

    let issues = untracked.len() + needs_analysis.len() + stale.len();
    if issues == 0 {
        println!("\n{}", "✓ Portfolio is healthy!".green());
    }

    Ok(())
}

