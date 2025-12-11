mod data;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use data::{
    analysis_dir, analysis_file_path, check_project_dirty, chrono_lite, detect_untracked_projects,
    find_ideas_repo, get_project_head_commit, get_recent_activity, has_analysis_file,
    load_analysis_meta, load_dotfiles, load_ideas, load_plans, load_projects, save_analysis_meta,
    Project, ProjectAnalysisMeta,
};

#[derive(Parser)]
#[command(name = "icli")]
#[command(about = "Unified CLI for ideas, projects, and plans")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List and search ideas from the ideas repo
    Ideas {
        /// Filter by status (active, dormant, unknown)
        #[arg(short, long)]
        status: Option<String>,

        /// Search term
        #[arg(short = 'q', long)]
        search: Option<String>,
    },

    /// List and search projects from the inventory
    Projects {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Search term
        #[arg(short = 'q', long)]
        search: Option<String>,

        /// Group by category
        #[arg(short, long)]
        group: bool,

        /// Only show projects with analysis files
        #[arg(short, long)]
        analyzed: bool,

        /// Show analysis status column
        #[arg(long)]
        show_analysis: bool,
    },

    /// Show analysis summary for a project (quick preview)
    Summary {
        /// Project name
        project: String,
    },

    /// List and search Claude plans
    Plans {
        /// Search term
        #[arg(short = 'q', long)]
        search: Option<String>,
    },

    /// Search across all sources (ideas, projects, plans)
    Search {
        /// Search term
        query: String,
    },

    /// Show statistics across all sources
    Stats,

    /// Refresh the project inventory cache
    Refresh,

    /// List DX tools and configs from dotfiles
    Dotfiles {
        /// Filter by category (dx-script, dx-tool, shell-config, app-config, tool-list, claude-skill)
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Show projects that need (re-)analysis
    Dirty {
        /// Only show projects that already have analysis files
        #[arg(long)]
        tracked_only: bool,

        /// Only show projects with commits since last analysis
        #[arg(long)]
        stale_only: bool,
    },

    /// Generate analysis for a project
    Analyze {
        /// Project name to analyze
        project: String,

        /// Only generate summary section (faster)
        #[arg(long)]
        summary_only: bool,

        /// Force re-analysis even if not dirty
        #[arg(long)]
        force: bool,

        /// Use Claude CLI to fill in analysis content (slower but richer)
        #[arg(long)]
        deep: bool,
    },

    /// Output analysis file content for a project
    Context {
        /// Project name
        project: String,
    },

    /// Create a zip bundle for NotebookLM upload
    Snapshot {
        /// Output path (defaults to ~/Downloads/ideas-snapshot-YYYY-MM-DD.zip)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show portfolio health: new projects, stale analyses, recent activity
    Status,

    /// Remove orphaned analysis files (no matching project in inventory)
    Prune {
        /// Actually delete files (default is dry-run)
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ideas { status, search } => cmd_ideas(status, search),
        Commands::Projects {
            category,
            search,
            group,
            analyzed,
            show_analysis,
        } => cmd_projects(category, search, group, analyzed, show_analysis),
        Commands::Summary { project } => cmd_summary(&project),
        Commands::Plans { search } => cmd_plans(search),
        Commands::Search { query } => cmd_search(&query),
        Commands::Stats => cmd_stats(),
        Commands::Refresh => cmd_refresh(),
        Commands::Dotfiles { category } => cmd_dotfiles(category),
        Commands::Dirty {
            tracked_only,
            stale_only,
        } => cmd_dirty(tracked_only, stale_only),
        Commands::Analyze {
            project,
            summary_only,
            force,
            deep,
        } => cmd_analyze(&project, summary_only, force, deep),
        Commands::Context { project } => cmd_context(&project),
        Commands::Snapshot { output } => cmd_snapshot(output),
        Commands::Status => cmd_status(),
        Commands::Prune { force } => cmd_prune(force),
    }
}

fn cmd_ideas(status: Option<String>, search: Option<String>) -> Result<()> {
    let ideas = load_ideas(&find_ideas_repo()?)?;

    let filtered: Vec<_> = ideas
        .iter()
        .filter(|i| {
            if let Some(ref s) = status {
                if i.status != *s {
                    return false;
                }
            }
            if let Some(ref q) = search {
                let q = q.to_lowercase();
                if !i.folder.to_lowercase().contains(&q)
                    && !i.description.to_lowercase().contains(&q)
                    && !i.tags.iter().any(|t| t.to_lowercase().contains(&q))
                {
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

fn cmd_projects(
    category: Option<String>,
    search: Option<String>,
    group: bool,
    analyzed_only: bool,
    show_analysis: bool,
) -> Result<()> {
    let projects = load_projects()?;

    let filtered: Vec<_> = projects
        .iter()
        .filter(|p| {
            // Filter by analyzed status
            if analyzed_only && !has_analysis_file(&p.name) {
                return false;
            }
            if let Some(ref c) = category {
                if p.category != *c {
                    return false;
                }
            }
            if let Some(ref q) = search {
                let q = q.to_lowercase();
                if !p.name.to_lowercase().contains(&q)
                    && !p.description.to_lowercase().contains(&q)
                    && !p.category.to_lowercase().contains(&q)
                    && !p.tech.to_lowercase().contains(&q)
                {
                    return false;
                }
            }
            true
        })
        .collect();

    if group {
        // Group by category
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
                let analyzed = if has_analysis_file(&p.name) {
                    "[A]".green()
                } else {
                    "[ ]".dimmed()
                };
                if show_analysis {
                    println!(
                        "  {} {:<18} {:<12} {}",
                        analyzed,
                        p.name,
                        &p.last_commit,
                        truncate(&p.description, 30)
                    );
                } else {
                    println!(
                        "  {:<20} {:<12} {}",
                        p.name,
                        &p.last_commit,
                        truncate(&p.description, 35)
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

            if show_analysis {
                let analyzed = if has_analysis_file(&p.name) {
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
                    truncate(&p.description, 30)
                );
            } else {
                println!(
                    "{:<22} {:<12} {:<12} {:<10} {}",
                    truncate(&p.name, 21),
                    cat_colored,
                    &p.last_commit,
                    &p.source,
                    truncate(&p.description, 30)
                );
            }
        }
    }

    Ok(())
}

fn cmd_plans(search: Option<String>) -> Result<()> {
    let plans = load_plans()?;

    let filtered: Vec<_> = plans
        .iter()
        .filter(|p| {
            if let Some(ref q) = search {
                let q = q.to_lowercase();
                if !p.name.to_lowercase().contains(&q) && !p.title.to_lowercase().contains(&q) {
                    return false;
                }
            }
            true
        })
        .collect();

    println!(
        "{:<35} {:<12} {}",
        "NAME".bold(),
        "MODIFIED".bold(),
        "TITLE".bold()
    );

    for plan in filtered {
        println!(
            "{:<35} {:<12} {}",
            truncate(&plan.name, 34),
            &plan.modified,
            truncate(&plan.title, 45)
        );
    }

    Ok(())
}

fn cmd_search(query: &str) -> Result<()> {
    let q = query.to_lowercase();

    // Search ideas
    let ideas = load_ideas(&find_ideas_repo()?)?;
    let idea_matches: Vec<_> = ideas
        .iter()
        .filter(|i| {
            i.folder.to_lowercase().contains(&q)
                || i.description.to_lowercase().contains(&q)
                || i.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect();

    // Search projects
    let projects = load_projects()?;
    let project_matches: Vec<_> = projects
        .iter()
        .filter(|p| {
            p.name.to_lowercase().contains(&q)
                || p.description.to_lowercase().contains(&q)
                || p.category.to_lowercase().contains(&q)
        })
        .collect();

    // Search plans
    let plans = load_plans()?;
    let plan_matches: Vec<_> = plans
        .iter()
        .filter(|p| p.name.to_lowercase().contains(&q) || p.title.to_lowercase().contains(&q))
        .collect();

    // Search dotfiles
    let dotfiles = load_dotfiles()?;
    let dotfile_matches: Vec<_> = dotfiles
        .iter()
        .filter(|d| {
            d.name.to_lowercase().contains(&q)
                || d.description.to_lowercase().contains(&q)
                || d.category.to_lowercase().contains(&q)
        })
        .collect();

    // Print results
    if !idea_matches.is_empty() {
        println!("{}", format!("\n=== Ideas ({}) ===", idea_matches.len()).yellow());
        for i in &idea_matches {
            println!("  {:<20} [{}] {}", i.folder, i.status, truncate(&i.description, 40));
        }
    }

    if !project_matches.is_empty() {
        println!(
            "{}",
            format!("\n=== Projects ({}) ===", project_matches.len()).cyan()
        );
        for p in &project_matches {
            println!(
                "  {:<20} [{}] {}",
                p.name,
                p.category,
                truncate(&p.description, 40)
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
            println!("  {:<20} [{}] {}", d.name, d.category, truncate(&d.description, 40));
        }
    }

    let total = idea_matches.len() + project_matches.len() + plan_matches.len() + dotfile_matches.len();
    println!(
        "\n{} total matches for '{}'",
        total.to_string().bold(),
        query
    );

    Ok(())
}

fn cmd_stats() -> Result<()> {
    let ideas = load_ideas(&find_ideas_repo()?)?;
    let projects = load_projects()?;
    let plans = load_plans()?;
    let dotfiles = load_dotfiles()?;

    println!("{}", "=== Portfolio Stats ===".bold());
    println!();

    // Ideas stats
    let active = ideas.iter().filter(|i| i.status == "active").count();
    let dormant = ideas.iter().filter(|i| i.status == "dormant").count();
    println!("{}: {} ({} active, {} dormant)", "Ideas".yellow(), ideas.len(), active, dormant);

    // Projects stats with analysis count
    let analysis_meta = load_analysis_meta().unwrap_or_default();
    let analyzed_count = analysis_meta.projects.len();
    println!("{}: {} ({} analyzed)", "Projects".cyan(), projects.len(), analyzed_count);
    let mut cat_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for p in &projects {
        *cat_counts.entry(&p.category).or_default() += 1;
    }
    let mut cats: Vec<_> = cat_counts.iter().collect();
    cats.sort_by(|a, b| b.1.cmp(a.1));
    for (cat, count) in cats.iter().take(5) {
        println!("  {}: {}", cat, count);
    }

    // Plans stats
    println!("{}: {}", "Plans".magenta(), plans.len());

    // Dotfiles stats
    let mut dx_cats: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for d in &dotfiles {
        *dx_cats.entry(&d.category).or_default() += 1;
    }
    let dx_summary: Vec<_> = dx_cats.iter().map(|(c, n)| format!("{} {}", n, c)).collect();
    println!("{}: {} ({})", "Dotfiles".blue(), dotfiles.len(), dx_summary.join(", "));

    println!();
    println!(
        "{}: {}",
        "Total items".bold(),
        ideas.len() + projects.len() + plans.len() + dotfiles.len()
    );

    Ok(())
}

fn cmd_status() -> Result<()> {
    println!("{}", "=== Portfolio Status ===".bold());
    println!();

    // 1. Untracked projects
    let untracked = detect_untracked_projects()?;
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

    // 2. Projects needing analysis (no analysis file yet)
    let projects = load_projects()?;
    let meta = load_analysis_meta()?;
    let mut needs_analysis: Vec<_> = projects
        .iter()
        .filter(|p| !has_analysis_file(&p.name) && p.commits > 0)
        .map(|p| (p.name.clone(), p.commits, p.category.clone()))
        .collect();
    needs_analysis.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by commit count

    if !needs_analysis.is_empty() {
        println!("{}", format!("⚠ Needs Analysis ({} projects):", needs_analysis.len()).yellow());
        for (name, commits, category) in needs_analysis.iter().take(5) {
            println!(
                "  {:<18} {} commits  [{}]",
                name.cyan(),
                commits,
                category
            );
        }
        if needs_analysis.len() > 5 {
            println!("  ... {} more (run {})", needs_analysis.len() - 5, "icli dirty".green());
        }
        println!();
    }

    // 3. Stale analyses (have analysis but outdated)
    let mut stale: Vec<_> = projects
        .iter()
        .filter_map(|p| {
            if !has_analysis_file(&p.name) {
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
        println!("{}", format!("⚠ Stale Analyses ({} total):", stale.len()).yellow());
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

    // 3. Recent activity
    let recent = get_recent_activity(7)?;
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

    // 4. Quick stats
    let ideas = load_ideas(&find_ideas_repo()?)?;
    let analyzed_count = meta.projects.len();
    println!(
        "{}: {} projects │ {} analyzed │ {} ideas",
        "Quick Stats".bold(),
        projects.len(),
        analyzed_count,
        ideas.len()
    );

    // Overall health indicator
    let issues = untracked.len() + needs_analysis.len() + stale.len();
    if issues == 0 {
        println!("\n{}", "✓ Portfolio is healthy!".green());
    }

    Ok(())
}

fn cmd_refresh() -> Result<()> {
    println!("Refreshing project inventory...");

    let home = dirs::home_dir().expect("No home directory");
    let scan_script = home.join("dotfiles/scripts/ideas/mq/projects-scan.sh");

    let status = std::process::Command::new("bash")
        .arg(&scan_script)
        .status()?;

    if status.success() {
        println!("{}", "Done!".green());
    } else {
        println!("{}", "Failed to refresh inventory".red());
    }

    Ok(())
}

fn cmd_dotfiles(category: Option<String>) -> Result<()> {
    let items = load_dotfiles()?;

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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

fn cmd_dirty(_tracked_only: bool, _stale_only: bool) -> Result<()> {
    let projects = load_projects()?;
    let meta = load_analysis_meta()?;

    // Collect stale (has analysis but outdated) and never analyzed
    let mut stale: Vec<(String, u32, String)> = Vec::new();
    let mut never: Vec<(String, u32, String)> = Vec::new();

    for project in &projects {
        let has_file = has_analysis_file(&project.name);

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

    // Sort by commits descending
    stale.sort_by(|a, b| b.1.cmp(&a.1));
    never.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{}", "=== Analysis Status ===".bold());
    println!();

    // Stale analyses
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

    // Never analyzed
    if !never.is_empty() {
        println!("{}", format!("Never Analyzed ({} projects):", never.len()).cyan());
        for (name, commits, category) in never.iter().take(10) {
            println!(
                "  {:<22} {} commits  [{}]",
                name,
                commits,
                category
            );
        }
        if never.len() > 10 {
            println!("  ... {} more", never.len() - 10);
        }
        println!();
    }

    // Summary
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

fn cmd_analyze(project_name: &str, summary_only: bool, force: bool, deep: bool) -> Result<()> {
    let projects = load_projects()?;
    let meta = load_analysis_meta()?;

    // Find the project
    let project = projects
        .iter()
        .find(|p| p.name == project_name)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found in inventory", project_name))?;

    // Check if analysis is needed
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

    // Run the analysis script
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
        // Update meta
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

        save_analysis_meta(&meta)?;

        println!("{}", "Analysis complete!".green());
        let analysis_path = analysis_file_path(project_name)?;
        println!("  Output: {}", analysis_path.display());
    } else {
        println!("{}", "Analysis failed".red());
    }

    Ok(())
}

fn cmd_context(project_name: &str) -> Result<()> {
    let analysis_path = analysis_file_path(project_name)?;

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

fn cmd_summary(project_name: &str) -> Result<()> {
    let analysis_path = analysis_file_path(project_name)?;

    if !analysis_path.exists() {
        eprintln!(
            "{}: No analysis file for '{}'",
            "Error".red(),
            project_name
        );
        eprintln!("Run: icli analyze {} --deep", project_name);
        std::process::exit(1);
    }

    let content = std::fs::read_to_string(&analysis_path)?;

    // Extract just the Summary section (up to "---" or "## Deep Dive")
    let mut in_summary = false;
    let mut output = String::new();

    for line in content.lines() {
        // Start capturing at the title or Summary header
        if line.starts_with("# ") || line.starts_with("> ") {
            in_summary = true;
        }

        // Stop at Deep Dive section or horizontal rule
        if line == "---" || line.starts_with("## Deep Dive") {
            break;
        }

        if in_summary {
            output.push_str(line);
            output.push('\n');
        }
    }

    print_markdown(&output);
    Ok(())
}

/// Render markdown to terminal with colors and max width
fn print_markdown(content: &str) {
    use termimad::{MadSkin, terminal_size, Area};

    let skin = MadSkin::default();

    // Get terminal width, cap at 100 for readability
    let term_width = terminal_size().0.min(100);

    // Create an area with the desired width
    let area = Area::new(0, 0, term_width, 1000);

    // Get formatted text for the area width
    let text = skin.area_text(content, &area);
    print!("{}", text);
}

/// Get current timestamp in ISO format
fn chrono_now() -> String {
    // Simple ISO 8601 format without external crate
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let secs = now.as_secs() as i64;

    // Reuse existing chrono_lite function
    let date = chrono_lite(secs);

    // Add time component (simplified)
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    format!("{}T{:02}:{:02}:{:02}", date, hours, minutes, seconds)
}


fn cmd_snapshot(output: Option<String>) -> Result<()> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    let home = dirs::home_dir().expect("No home directory");
    let ideas_root = home.join("Developer/ideas");

    // Determine output path
    let today = chrono_lite(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    );
    let default_name = format!("ideas-snapshot-{}.zip", today);
    let output_path = match output {
        Some(p) => std::path::PathBuf::from(p),
        None => home.join("Downloads").join(&default_name),
    };

    println!("{}", "Creating NotebookLM snapshot...".cyan());

    // Create zip
    let file = std::fs::File::create(&output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. Analysis files (already markdown)
    let analysis_dir = ideas_root.join("_data/analysis");
    let mut analysis_count = 0;
    if analysis_dir.exists() {
        for entry in std::fs::read_dir(&analysis_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") {
                let name = format!("analysis/{}", path.file_name().unwrap().to_string_lossy());
                let content = std::fs::read(&path)?;
                zip.start_file(&name, options)?;
                zip.write_all(&content)?;
                analysis_count += 1;
            }
        }
    }

    // 2. Project inventory (JSON → Markdown)
    let inventory_path = ideas_root.join("_data/project-inventory.json");
    if inventory_path.exists() {
        let md = inventory_to_markdown(&inventory_path)?;
        zip.start_file("project-inventory.md", options)?;
        zip.write_all(md.as_bytes())?;
    }

    // 3. Tracker (CSV → Markdown)
    let tracker_path = ideas_root.join("_tracker.csv");
    if tracker_path.exists() {
        let md = tracker_to_markdown(&tracker_path)?;
        zip.start_file("ideas-tracker.md", options)?;
        zip.write_all(md.as_bytes())?;
    }

    // 4. Repo docs (already markdown)
    let readme_path = ideas_root.join("README.md");
    if readme_path.exists() {
        let content = std::fs::read(&readme_path)?;
        zip.start_file("README.md", options)?;
        zip.write_all(&content)?;
    }

    let claude_md_path = ideas_root.join("CLAUDE.md");
    if claude_md_path.exists() {
        let content = std::fs::read(&claude_md_path)?;
        zip.start_file("CLAUDE.md", options)?;
        zip.write_all(&content)?;
    }

    zip.finish()?;

    // Print summary
    let zip_size = std::fs::metadata(&output_path)?.len();
    println!("  {} analysis files", analysis_count.to_string().bold());
    println!("  {} project inventory (as markdown)", "1".bold());
    println!("  {} tracker (as markdown)", "1".bold());
    println!("  {} repo docs", "2".bold());
    println!();
    println!(
        "{}: {} ({} compressed)",
        "Snapshot".green(),
        output_path.display(),
        format_size(zip_size)
    );

    Ok(())
}

/// Convert project-inventory.json to markdown
fn inventory_to_markdown(path: &std::path::Path) -> Result<String> {
    #[derive(serde::Deserialize)]
    struct Inventory {
        projects: Vec<Project>,
    }

    let content = std::fs::read_to_string(path)?;
    let inv: Inventory = serde_json::from_str(&content)?;

    let mut md = String::from("# Project Inventory\n\n");
    md.push_str(&format!("Total projects: {}\n\n", inv.projects.len()));

    // Group by category
    let mut by_cat: std::collections::HashMap<&str, Vec<&Project>> = std::collections::HashMap::new();
    for p in &inv.projects {
        by_cat.entry(&p.category).or_default().push(p);
    }

    let mut cats: Vec<_> = by_cat.keys().collect();
    cats.sort();

    for cat in cats {
        let projects = &by_cat[cat];
        md.push_str(&format!("## {} ({} projects)\n\n", cat, projects.len()));

        for p in projects {
            md.push_str(&format!("### {}\n\n", p.name));
            md.push_str(&format!("- **Path**: {}\n", p.path));
            md.push_str(&format!("- **Tech**: {}\n", p.tech));
            md.push_str(&format!("- **Last commit**: {}\n", p.last_commit));
            md.push_str(&format!("- **Description**: {}\n\n", p.description));
        }
    }

    Ok(md)
}

/// Convert _tracker.csv to markdown
fn tracker_to_markdown(path: &std::path::Path) -> Result<String> {
    let mut md = String::from("# Ideas Tracker\n\n");
    md.push_str("| Folder | Tags | Description | Created | Modified | Sessions |\n");
    md.push_str("|--------|------|-------------|---------|----------|----------|\n");

    let mut reader = csv::Reader::from_path(path)?;
    for result in reader.records() {
        let record = result?;
        let folder = record.get(0).unwrap_or("");
        let tags = record.get(1).unwrap_or("").trim_matches('"');
        let description = record.get(2).unwrap_or("").trim_matches('"');
        let created = record.get(3).unwrap_or("");
        let modified = record.get(4).unwrap_or("");
        let sessions = record.get(5).unwrap_or("0");

        if !folder.is_empty() {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                folder, tags, description, created, modified, sessions
            ));
        }
    }

    Ok(md)
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn cmd_prune(force: bool) -> Result<()> {

    let projects = load_projects()?;
    let project_names: std::collections::HashSet<_> = projects.iter().map(|p| p.name.as_str()).collect();

    let analysis_path = analysis_dir()?;
    let mut orphans: Vec<(String, std::path::PathBuf)> = Vec::new();

    // Find analysis files with no matching project
    if analysis_path.exists() {
        for entry in std::fs::read_dir(&analysis_path)? {
            let entry = entry?;
            let path = entry.path();

            // Skip non-md files and _meta.json
            if path.extension().map_or(true, |e| e != "md") {
                continue;
            }

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");

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
        // Delete orphaned files and remove from meta
        let mut meta = load_analysis_meta()?;

        for (name, path) in &orphans {
            std::fs::remove_file(path)?;
            meta.projects.remove(name);
            println!("  {} {}", "Deleted:".red(), name);
        }

        save_analysis_meta(&meta)?;
        println!();
        println!(
            "{}: Removed {} orphaned files",
            "Done".green(),
            orphans.len()
        );
    } else {
        println!(
            "Run {} to delete these files",
            "icli prune --force".cyan()
        );
    }

    Ok(())
}
