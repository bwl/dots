//! CLI-specific data types and loading functions.
//!
//! All core types and functions are re-exported from ideas-core.

// Re-export everything from ideas-core
pub use ideas_core::{
    // Ideas
    load_ideas, Idea,
    // Projects
    load_projects, Project,
    // Plans
    load_plans, Plan,
    // Dotfiles
    load_dotfiles, DxItem,
    // Path resolution
    analysis_dir, analysis_file_path, find_ideas_repo,
    // Analysis
    has_analysis_file, load_analysis_summary,
    load_analysis_meta, save_analysis_meta, AnalysisMeta, ProjectAnalysisMeta,
    // Dirty detection
    check_project_dirty, DirtyProject,
    // Project discovery
    detect_untracked_projects, UntrackedProject,
    // Recent activity
    get_recent_activity, RecentProject,
    // Git operations
    count_commits_since, get_project_head_commit,
    // Utilities
    chrono_lite,
};
