//! Domain-level search helpers shared between CLI and TUI.

use crate::{DxItem, Idea, Plan, Project};

/// Normalize a user query for matching.
pub fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}

/// True if an idea matches the (lowercased) query.
pub fn idea_matches_query(idea: &Idea, query_lower: &str) -> bool {
    if query_lower.is_empty() {
        return true;
    }
    idea.folder.to_lowercase().contains(query_lower)
        || idea.description.to_lowercase().contains(query_lower)
        || idea.tags.iter().any(|t| t.to_lowercase().contains(query_lower))
}

/// True if a project matches the (lowercased) query.
pub fn project_matches_query(project: &Project, query_lower: &str) -> bool {
    if query_lower.is_empty() {
        return true;
    }
    project.name.to_lowercase().contains(query_lower)
        || project.summary.to_lowercase().contains(query_lower)
        || project.description.to_lowercase().contains(query_lower)
        || project.category.to_lowercase().contains(query_lower)
        || project.tech.to_lowercase().contains(query_lower)
}

/// True if a plan matches the (lowercased) query.
pub fn plan_matches_query(plan: &Plan, query_lower: &str) -> bool {
    if query_lower.is_empty() {
        return true;
    }
    plan.name.to_lowercase().contains(query_lower) || plan.title.to_lowercase().contains(query_lower)
}

/// True if a DX item matches the (lowercased) query.
pub fn dxitem_matches_query(item: &DxItem, query_lower: &str) -> bool {
    if query_lower.is_empty() {
        return true;
    }
    item.name.to_lowercase().contains(query_lower)
        || item.description.to_lowercase().contains(query_lower)
        || item.category.to_lowercase().contains(query_lower)
}

