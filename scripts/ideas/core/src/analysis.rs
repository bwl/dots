//! Analysis files and metadata.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::git::{count_commits_since, get_project_head_commit};
use crate::paths::{analysis_file_path_with_paths, IdeasPaths};
use crate::projects::Project;

/// Load the summary section from an analysis file.
pub fn load_analysis_summary_with_paths(
    paths: &IdeasPaths,
    project_name: &str,
) -> Option<String> {
    let path = analysis_file_path_with_paths(paths, project_name);

    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&path).ok()?;

    let mut in_summary = false;
    let mut output = String::new();

    for line in content.lines() {
        if line.starts_with("# ") || line.starts_with("> ") {
            in_summary = true;
        }
        if line == "---" || line.starts_with("## Deep Dive") {
            break;
        }
        if in_summary {
            output.push_str(line);
            output.push('\n');
        }
    }

    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

/// Load the summary section from an analysis file.
pub fn load_analysis_summary(project_name: &str) -> Option<String> {
    let paths = IdeasPaths::detect().ok()?;
    load_analysis_summary_with_paths(&paths, project_name)
}

// ============ Analysis Metadata ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysisMeta {
    pub analyzed_at: String,
    pub analyzed_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMeta {
    pub version: u32,
    pub projects: HashMap<String, ProjectAnalysisMeta>,
}

impl Default for AnalysisMeta {
    fn default() -> Self {
        Self {
            version: 1,
            projects: HashMap::new(),
        }
    }
}

/// Load analysis metadata from _meta.json.
pub fn load_analysis_meta_with_paths(paths: &IdeasPaths) -> Result<AnalysisMeta> {
    let meta_path = &paths.analysis_meta_path;

    if !meta_path.exists() {
        return Ok(AnalysisMeta::default());
    }

    let content = std::fs::read_to_string(meta_path)?;
    let meta: AnalysisMeta = serde_json::from_str(&content)?;

    Ok(meta)
}

/// Load analysis metadata from _meta.json.
pub fn load_analysis_meta() -> Result<AnalysisMeta> {
    let paths = IdeasPaths::detect()?;
    load_analysis_meta_with_paths(&paths)
}

/// Save analysis metadata to _meta.json.
pub fn save_analysis_meta_with_paths(paths: &IdeasPaths, meta: &AnalysisMeta) -> Result<()> {
    let meta_path = &paths.analysis_meta_path;

    let content = serde_json::to_string_pretty(meta)?;
    std::fs::write(meta_path, content)?;

    Ok(())
}

/// Save analysis metadata to _meta.json.
pub fn save_analysis_meta(meta: &AnalysisMeta) -> Result<()> {
    let paths = IdeasPaths::detect()?;
    save_analysis_meta_with_paths(&paths, meta)
}

// ============ Dirty Detection ============

#[derive(Debug, Clone)]
pub struct DirtyProject {
    pub name: String,
    pub path: String,
    pub analyzed_at: Option<String>,
    pub analyzed_commit: Option<String>,
    pub current_commit: Option<String>,
    pub commits_since: Option<u32>,
}

/// Check if a project is dirty (needs re-analysis).
pub fn check_project_dirty(project: &Project, meta: &AnalysisMeta) -> DirtyProject {
    let analysis_meta = meta.projects.get(&project.name);
    let current_commit = get_project_head_commit(&project.path);

    let (analyzed_at, analyzed_commit, commits_since) = match analysis_meta {
        Some(m) => {
            let since = current_commit
                .as_ref()
                .and_then(|_| count_commits_since(&project.path, &m.analyzed_commit));
            (
                Some(m.analyzed_at.clone()),
                Some(m.analyzed_commit.clone()),
                since,
            )
        }
        None => (None, None, None),
    };

    DirtyProject {
        name: project.name.clone(),
        path: project.path.clone(),
        analyzed_at,
        analyzed_commit,
        current_commit,
        commits_since,
    }
}
