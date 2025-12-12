use anyhow::Result;
use colored::Colorize;

use crate::data::{load_plans_with_paths, normalize_query, plan_matches_query, IdeasPaths};
use super::util::truncate;

pub fn cmd_plans(paths: &IdeasPaths, search: Option<String>) -> Result<()> {
    let plans = load_plans_with_paths(paths)?;
    let q_lower = search.as_deref().map(normalize_query);

    let filtered: Vec<_> = plans
        .iter()
        .filter(|p| {
            if let Some(ref q) = q_lower {
                if !plan_matches_query(p, q) {
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

