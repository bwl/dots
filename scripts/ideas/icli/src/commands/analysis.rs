use anyhow::Result;
use colored::Colorize;

use crate::data::{
    analysis_dir_with_paths, analysis_file_path_with_paths, check_project_dirty, chrono_now,
    get_project_head_commit, has_analysis_file_with_paths, load_analysis_meta_with_paths,
    load_analysis_summary_with_paths, load_projects_with_paths, save_analysis_meta_with_paths,
    IdeasPaths, ProjectAnalysisMeta,
};
use super::util::print_markdown;

pub fn cmd_dirty(paths: &IdeasPaths, _tracked_only: bool, _stale_only: bool) -> Result<()> {
    let projects = load_projects_with_paths(paths)?;
    let meta = load_analysis_meta_with_paths(paths)?;

    let mut stale: Vec<(String, u32, String)> = Vec::new();
    let mut never: Vec<(String, u32, String)> = Vec::new();

    for project in &projects {
        let has_file = has_analysis_file_with_paths(paths, &project.name);

        if has_file {
            let dirty = check_project_dirty(project, &meta);
            let commits = dirty.commits_since.unwrap_or(0);
            if commits > 0 {
                stale.push((project.name.clone(), commits, project.category.clone()));
            }
        } else if project.commits > 0 {
            never.push((project.name.clone(), project.commits, project.category.clone()));
        }
    }

    stale.sort_by(|a, b| b.1.cmp(&a.1));
    never.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{}", "=== Analysis Status ===".bold());
    println!();

    if !stale.is_empty() {
        println!("{}", format!("Stale ({} projects):", stale.len()).yellow());
        for (name, commits, category) in &stale {
            println!(
                "  {:<22} {} commits since  [{}]",
                name.cyan(),
                commits.to_string().red(),
                category
            );
        }
        println!();
    }

    if !never.is_empty() {
        println!("{}", format!("Never Analyzed ({} projects):", never.len()).cyan());
        for (name, commits, category) in never.iter().take(10) {
            println!("  {:<22} {} commits  [{}]", name, commits, category);
        }
        if never.len() > 10 {
            println!("  ... {} more", never.len() - 10);
        }
        println!();
    }

    let analyzed_count = meta.projects.len();
    let up_to_date = analyzed_count - stale.len();

    println!(
        "{}: {} analyzed ({} up to date, {} stale), {} never analyzed",
        "Summary".bold(),
        analyzed_count,
        up_to_date.to_string().green(),
        stale.len().to_string().yellow(),
        never.len().to_string().cyan()
    );

    if stale.is_empty() && never.is_empty() {
        println!("\n{}", "✓ All projects are analyzed and up to date!".green());
    }

    Ok(())
}

pub fn cmd_analyze(
    paths: &IdeasPaths,
    project_name: &str,
    summary_only: bool,
    force: bool,
    deep: bool,
) -> Result<()> {
    let projects = load_projects_with_paths(paths)?;
    let meta = load_analysis_meta_with_paths(paths)?;

    let project = projects
        .iter()
        .find(|p| p.name == project_name)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found in inventory", project_name))?;

    let dirty = check_project_dirty(project, &meta);
    let needs_analysis = force
        || dirty.analyzed_at.is_none()
        || dirty.commits_since.unwrap_or(0) > 0;

    if !needs_analysis {
        println!(
            "Project '{}' is up to date (analyzed at {}, commit {})",
            project_name,
            dirty.analyzed_at.as_deref().unwrap_or("-"),
            dirty.analyzed_commit.as_deref().unwrap_or("-")
        );
        println!("Use --force to re-analyze anyway.");
        return Ok(());
    }

    let mode = if deep {
        "deep (Claude-powered)"
    } else if summary_only {
        "summary only"
    } else {
        "scaffold"
    };

    println!("{}", format!("Analyzing {}...", project_name).cyan());
    println!("  Path: {}", project.path);
    println!("  Tech: {}", project.tech);
    println!("  Mode: {}", mode);
    println!();

    let home = dirs::home_dir().expect("No home directory");
    let script_name = if deep {
        "analyze-project-deep.sh"
    } else {
        "analyze-project.sh"
    };
    let analyze_script = home.join(format!("dotfiles/scripts/ideas/mq/{}", script_name));

    let mut cmd = std::process::Command::new("bash");
    cmd.arg(&analyze_script).arg(&project.path);
    if summary_only {
        cmd.arg("--summary-only");
    }

    let status = cmd.status()?;

    if status.success() {
        let mut meta = meta;
        let current_commit = get_project_head_commit(&project.path).unwrap_or_default();
        let now = chrono_now();

        meta.projects.insert(
            project_name.to_string(),
            ProjectAnalysisMeta {
                analyzed_at: now,
                analyzed_commit: current_commit,
            },
        );

        save_analysis_meta_with_paths(paths, &meta)?;

        println!("{}", "Analysis complete!".green());
        let analysis_path = analysis_file_path_with_paths(paths, project_name);
        println!("  Output: {}", analysis_path.display());
    } else {
        println!("{}", "Analysis failed".red());
    }

    Ok(())
}

pub fn cmd_context(paths: &IdeasPaths, project_name: &str) -> Result<()> {
    let analysis_path = analysis_file_path_with_paths(paths, project_name);

    if !analysis_path.exists() {
        eprintln!(
            "{}: No analysis file for '{}'",
            "Error".red(),
            project_name
        );
        eprintln!("Run: icli analyze {}", project_name);
        std::process::exit(1);
    }

    let content = std::fs::read_to_string(&analysis_path)?;
    print_markdown(&content);

    Ok(())
}

pub fn cmd_summary(paths: &IdeasPaths, project_name: &str) -> Result<()> {
    if let Some(summary) = load_analysis_summary_with_paths(paths, project_name) {
        print_markdown(&summary);
        return Ok(());
    }

    eprintln!(
        "{}: No analysis file for '{}'",
        "Error".red(),
        project_name
    );
    eprintln!("Run: icli analyze {} --deep", project_name);
    std::process::exit(1);
}

pub fn cmd_prune(paths: &IdeasPaths, force: bool) -> Result<()> {
    let projects = load_projects_with_paths(paths)?;
    let project_names: std::collections::HashSet<_> =
        projects.iter().map(|p| p.name.as_str()).collect();

    let analysis_path = analysis_dir_with_paths(paths);
    let mut orphans: Vec<(String, std::path::PathBuf)> = Vec::new();

    if analysis_path.exists() {
        for entry in std::fs::read_dir(&analysis_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(true, |e| e != "md") {
                continue;
            }

            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

            if !project_names.contains(name) {
                orphans.push((name.to_string(), path));
            }
        }
    }

    if orphans.is_empty() {
        println!("{}", "✓ No orphaned analysis files found".green());
        return Ok(());
    }

    orphans.sort_by(|a, b| a.0.cmp(&b.0));

    println!(
        "{}",
        format!("Found {} orphaned analysis files:", orphans.len()).yellow()
    );
    println!();

    for (name, path) in &orphans {
        println!("  {} → {}", name.red(), path.display());
    }
    println!();

    if force {
        let mut meta = load_analysis_meta_with_paths(paths)?;

        for (name, path) in &orphans {
            std::fs::remove_file(path)?;
            meta.projects.remove(name);
            println!("  {} {}", "Deleted:".red(), name);
        }

        save_analysis_meta_with_paths(paths, &meta)?;
        println!();
        println!("{}: Removed {} orphaned files", "Done".green(), orphans.len());
    } else {
        println!("Run {} to delete these files", "icli prune --force".cyan());
    }

    Ok(())
}

