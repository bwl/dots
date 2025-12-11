//! TUI-specific data types and sorting functions.
//!
//! Core types and their loading functions are re-exported from ideas-core.
//! This module contains TUI-specific functionality like sorting options
//! and markdown file discovery.

use std::path::{Path, PathBuf};

// Re-export core types and functions
pub use ideas_core::{
    // Ideas
    find_ideas_repo, load_ideas, Idea,
    // Projects
    has_analysis_file, load_analysis_summary, load_projects, Project,
    // Plans
    load_plans, Plan,
    // Dotfiles
    load_dotfiles, DxItem,
    // Status/Analysis
    check_project_dirty, detect_untracked_projects, get_recent_activity, load_analysis_meta,
    RecentProject, UntrackedProject,
};

// ============ Idea Sorting ============

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Name,
    Status,
    Questions,
    Sessions,
    Modified,
}

impl SortBy {
    pub fn next(self) -> Self {
        match self {
            SortBy::Name => SortBy::Status,
            SortBy::Status => SortBy::Questions,
            SortBy::Questions => SortBy::Sessions,
            SortBy::Sessions => SortBy::Modified,
            SortBy::Modified => SortBy::Name,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortBy::Name => "name",
            SortBy::Status => "status",
            SortBy::Questions => "questions",
            SortBy::Sessions => "sessions",
            SortBy::Modified => "modified",
        }
    }
}

pub fn sort_ideas(ideas: &mut [Idea], sort_by: SortBy) {
    match sort_by {
        SortBy::Name => ideas.sort_by(|a, b| a.folder.cmp(&b.folder)),
        SortBy::Status => ideas.sort_by(|a, b| a.status.cmp(&b.status)),
        SortBy::Questions => {
            ideas.sort_by(|a, b| b.open_questions.len().cmp(&a.open_questions.len()))
        }
        SortBy::Sessions => ideas.sort_by(|a, b| b.sessions.cmp(&a.sessions)),
        SortBy::Modified => ideas.sort_by(|a, b| b.modified.cmp(&a.modified)),
    }
}

// ============ Markdown File Discovery ============

/// Find all markdown files in an idea folder, with README.md first.
pub fn find_markdown_files(idea_path: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(idea_path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "md"))
        .collect();

    // Sort with README.md first, then alphabetically
    files.sort_by(|a, b| {
        let a_readme = a.file_name().map_or(false, |n| n == "README.md");
        let b_readme = b.file_name().map_or(false, |n| n == "README.md");
        match (a_readme, b_readme) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        }
    });
    files
}

// ============ Project Sorting ============

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectSortBy {
    Name,
    Category,
    LastCommit,
    Analyzed,
}

impl ProjectSortBy {
    pub fn next(self) -> Self {
        match self {
            ProjectSortBy::Name => ProjectSortBy::Category,
            ProjectSortBy::Category => ProjectSortBy::LastCommit,
            ProjectSortBy::LastCommit => ProjectSortBy::Analyzed,
            ProjectSortBy::Analyzed => ProjectSortBy::Name,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ProjectSortBy::Name => "name",
            ProjectSortBy::Category => "category",
            ProjectSortBy::LastCommit => "last commit",
            ProjectSortBy::Analyzed => "analyzed",
        }
    }
}

pub fn sort_projects(projects: &mut [Project], sort_by: ProjectSortBy) {
    match sort_by {
        ProjectSortBy::Name => projects.sort_by(|a, b| a.name.cmp(&b.name)),
        ProjectSortBy::Category => projects.sort_by(|a, b| a.category.cmp(&b.category)),
        ProjectSortBy::LastCommit => projects.sort_by(|a, b| b.last_commit.cmp(&a.last_commit)),
        ProjectSortBy::Analyzed => projects.sort_by(|a, b| {
            let a_has = has_analysis_file(&a.name);
            let b_has = has_analysis_file(&b.name);
            b_has.cmp(&a_has)
        }),
    }
}

// ============ Plan Sorting ============

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanSortBy {
    Name,
    Modified,
}

impl PlanSortBy {
    pub fn next(self) -> Self {
        match self {
            PlanSortBy::Name => PlanSortBy::Modified,
            PlanSortBy::Modified => PlanSortBy::Name,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PlanSortBy::Name => "name",
            PlanSortBy::Modified => "modified",
        }
    }
}

pub fn sort_plans(plans: &mut [Plan], sort_by: PlanSortBy) {
    match sort_by {
        PlanSortBy::Name => plans.sort_by(|a, b| a.name.cmp(&b.name)),
        PlanSortBy::Modified => plans.sort_by(|a, b| b.modified.cmp(&a.modified)),
    }
}

// ============ Dotfiles Sorting ============

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DotfilesSortBy {
    Name,
    Category,
}

impl DotfilesSortBy {
    pub fn next(self) -> Self {
        match self {
            DotfilesSortBy::Name => DotfilesSortBy::Category,
            DotfilesSortBy::Category => DotfilesSortBy::Name,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DotfilesSortBy::Name => "name",
            DotfilesSortBy::Category => "category",
        }
    }
}

pub fn sort_dotfiles(items: &mut [DxItem], sort_by: DotfilesSortBy) {
    match sort_by {
        DotfilesSortBy::Name => items.sort_by(|a, b| a.name.cmp(&b.name)),
        DotfilesSortBy::Category => items.sort_by(|a, b| a.category.cmp(&b.category)),
    }
}

// ============ Search Result ============

#[derive(Debug, Clone)]
pub enum SearchSource {
    Ideas,
    Projects,
    Plans,
    Dotfiles,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub source: SearchSource,
    pub name: String,
    pub description: String,
    pub index: usize, // Index in the original collection
}
