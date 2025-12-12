mod commands;
mod data;

use anyhow::Result;
use clap::{Parser, Subcommand};

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

    /// Generate AI-powered 1-line summary for project(s)
    Summarize {
        /// Project name (or "all-missing" to generate for all projects without summaries)
        project: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = data::IdeasPaths::detect()?;

    match cli.command {
        Commands::Ideas { status, search } => {
            commands::ideas::cmd_ideas(&paths, status, search)
        }
        Commands::Projects {
            category,
            search,
            group,
            analyzed,
            show_analysis,
        } => commands::projects::cmd_projects(
            &paths,
            category,
            search,
            group,
            analyzed,
            show_analysis,
        ),
        Commands::Summary { project } => commands::analysis::cmd_summary(&paths, &project),
        Commands::Plans { search } => commands::plans::cmd_plans(&paths, search),
        Commands::Search { query } => commands::search::cmd_search(&paths, &query),
        Commands::Stats => commands::portfolio::cmd_stats(&paths),
        Commands::Refresh => commands::refresh::cmd_refresh(&paths),
        Commands::Dotfiles { category } => commands::dotfiles::cmd_dotfiles(&paths, category),
        Commands::Dirty {
            tracked_only,
            stale_only,
        } => commands::analysis::cmd_dirty(&paths, tracked_only, stale_only),
        Commands::Analyze {
            project,
            summary_only,
            force,
            deep,
        } => commands::analysis::cmd_analyze(&paths, &project, summary_only, force, deep),
        Commands::Context { project } => commands::analysis::cmd_context(&paths, &project),
        Commands::Snapshot { output } => commands::snapshot::cmd_snapshot(&paths, output),
        Commands::Status => commands::portfolio::cmd_status(&paths),
        Commands::Prune { force } => commands::analysis::cmd_prune(&paths, force),
        Commands::Summarize { project } => commands::summarize::cmd_summarize(&paths, &project),
    }
}

