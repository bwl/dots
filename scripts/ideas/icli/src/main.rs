mod data;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use data::{
    analysis_file_path, check_project_dirty, has_analysis_file, load_analysis_meta,
    load_dotfiles, load_ideas, load_plans, load_projects, save_analysis_meta, Project,
    ProjectAnalysisMeta,
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
    }
}

fn cmd_ideas(status: Option<String>, search: Option<String>) -> Result<()> {
    let ideas = load_ideas()?;

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
    let ideas = load_ideas()?;
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
    let ideas = load_ideas()?;
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

fn cmd_dirty(tracked_only: bool, stale_only: bool) -> Result<()> {
    let projects = load_projects()?;
    let meta = load_analysis_meta()?;

    let mut analyzed_dirty = Vec::new();
    let mut never_analyzed = Vec::new();

    for project in &projects {
        let dirty = check_project_dirty(project, &meta);
        let has_file = has_analysis_file(&project.name);

        if dirty.analyzed_at.is_some() && has_file {
            // Already analyzed - check if stale
            let commits = dirty.commits_since.unwrap_or(0);
            if commits > 0 || !stale_only {
                analyzed_dirty.push(dirty);
            }
        } else if !tracked_only {
            never_analyzed.push(dirty);
        }
    }

    println!("{}", "=== Dirty Projects ===".bold());
    println!();

    if !analyzed_dirty.is_empty() {
        println!("{}", "Analyzed (need refresh):".yellow());
        for d in &analyzed_dirty {
            let commits = d.commits_since.unwrap_or(0);
            let last = d.analyzed_at.as_deref().unwrap_or("-");
            if commits > 0 {
                println!(
                    "  {:<22} last: {}  {} since",
                    d.name,
                    last,
                    format!("{} commits", commits).red()
                );
            } else {
                println!("  {:<22} last: {}  (up to date)", d.name, last);
            }
        }
    }

    if !never_analyzed.is_empty() {
        println!();
        println!("{}", "Never analyzed:".cyan());
        // Group into one line if many
        if never_analyzed.len() > 10 {
            let names: Vec<_> = never_analyzed.iter().take(10).map(|d| d.name.as_str()).collect();
            println!(
                "  {}, ... ({} more)",
                names.join(", "),
                never_analyzed.len() - 10
            );
        } else {
            for d in &never_analyzed {
                println!("  {}", d.name);
            }
        }
    }

    let total_dirty = analyzed_dirty.iter().filter(|d| d.commits_since.unwrap_or(0) > 0).count();
    println!();
    println!(
        "{} stale, {} never analyzed",
        total_dirty.to_string().bold(),
        never_analyzed.len().to_string().bold()
    );

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
        let current_commit = data::get_project_head_commit(&project.path).unwrap_or_default();
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

fn chrono_lite(unix_secs: i64) -> String {
    // Convert unix timestamp to YYYY-MM-DD
    let days_since_epoch = unix_secs / 86400;
    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days in days_in_months.iter() {
        if remaining_days < *days {
            break;
        }
        remaining_days -= days;
        month += 1;
    }

    let day = remaining_days + 1;

    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
