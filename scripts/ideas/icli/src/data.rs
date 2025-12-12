//! CLI-specific data types and loading functions.
//!
//! All core types and functions are re-exported from ideas-core.

// Re-export everything from ideas-core
#[allow(unused_imports)]
pub use ideas_core::{
    // Ideas
    load_ideas, Idea,
    // Projects
    load_projects, load_projects_with_paths, Project,
    // Plans
    load_plans, load_plans_with_paths, Plan,
    // Dotfiles
    load_dotfiles, load_dotfiles_with_paths, DxItem,
    // Path resolution
    analysis_dir, analysis_dir_with_paths, analysis_file_path, analysis_file_path_with_paths,
    find_ideas_repo, find_ideas_repo_with_paths, has_analysis_file_with_paths, IdeasPaths,
    // Analysis
    has_analysis_file, load_analysis_summary, load_analysis_summary_with_paths,
    load_analysis_meta, load_analysis_meta_with_paths, save_analysis_meta,
    save_analysis_meta_with_paths, AnalysisMeta, ProjectAnalysisMeta,
    // Dirty detection
    check_project_dirty, DirtyProject,
    // Project discovery
    detect_untracked_projects, detect_untracked_projects_with_paths, UntrackedProject,
    // Recent activity
    get_recent_activity, get_recent_activity_with_paths, RecentProject,
    // Git operations
    count_commits_since, get_project_head_commit,
    // Search helpers
    dxitem_matches_query, idea_matches_query, normalize_query, plan_matches_query,
    project_matches_query,
    // Utilities
    chrono_lite, chrono_now,
};
