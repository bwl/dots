//! Shared data types and loading functions for the ideas ecosystem.
//!
//! This crate provides common functionality used by both `icli` and `ideas-tui`.

pub mod analysis;
pub mod dotfiles;
pub mod discovery;
pub mod git;
pub mod ideas;
pub mod paths;
pub mod plans;
pub mod projects;
pub mod recent;
pub mod search;
pub mod util;

// Re-export stable public API at the crate root.
pub use analysis::{
    check_project_dirty, load_analysis_meta, load_analysis_meta_with_paths,
    load_analysis_summary, load_analysis_summary_with_paths, save_analysis_meta,
    save_analysis_meta_with_paths, AnalysisMeta, DirtyProject, ProjectAnalysisMeta,
};
pub use dotfiles::{load_dotfiles, load_dotfiles_with_paths, DxItem};
pub use discovery::{detect_untracked_projects, detect_untracked_projects_with_paths, UntrackedProject};
pub use git::{count_commits_since, get_project_head_commit};
pub use ideas::{load_ideas, Idea};
pub use paths::{
    analysis_dir, analysis_dir_with_paths, analysis_file_path, analysis_file_path_with_paths,
    find_ideas_repo, find_ideas_repo_with_paths, has_analysis_file, has_analysis_file_with_paths,
    IdeasPaths,
};
pub use plans::{load_plans, load_plans_with_paths, Plan};
pub use projects::{load_projects, load_projects_with_paths, Project};
pub use recent::{get_recent_activity, get_recent_activity_with_paths, RecentProject};
pub use search::{
    dxitem_matches_query, idea_matches_query, normalize_query, plan_matches_query,
    project_matches_query,
};
pub use util::{chrono_lite, chrono_now};
