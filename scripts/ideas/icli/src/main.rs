mod data;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use data::{load_dotfiles, load_ideas, load_plans, load_projects, Project};

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ideas { status, search } => cmd_ideas(status, search),
        Commands::Projects { category, search, group } => cmd_projects(category, search, group),
        Commands::Plans { search } => cmd_plans(search),
        Commands::Search { query } => cmd_search(&query),
        Commands::Stats => cmd_stats(),
        Commands::Refresh => cmd_refresh(),
        Commands::Dotfiles { category } => cmd_dotfiles(category),
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

fn cmd_projects(category: Option<String>, search: Option<String>, group: bool) -> Result<()> {
    let projects = load_projects()?;

    let filtered: Vec<_> = projects
        .iter()
        .filter(|p| {
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
                println!(
                    "  {:<20} {:<12} {}",
                    p.name,
                    &p.last_commit,
                    truncate(&p.description, 35)
                );
            }
        }
    } else {
        println!(
            "{:<22} {:<12} {:<12} {:<10} {}",
            "NAME".bold(),
            "CATEGORY".bold(),
            "LAST_COMMIT".bold(),
            "SOURCE".bold(),
            "DESCRIPTION".bold()
        );

        for p in filtered {
            let cat_colored = match p.category.as_str() {
                "roguelike" => p.category.red(),
                "writing" => p.category.magenta(),
                "knowledge" => p.category.blue(),
                "simulation" => p.category.green(),
                "tui" | "cli" => p.category.cyan(),
                _ => p.category.white(),
            };

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

    // Projects stats
    println!("{}: {}", "Projects".cyan(), projects.len());
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
