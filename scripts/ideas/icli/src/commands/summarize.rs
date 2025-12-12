use anyhow::Result;
use colored::Colorize;

use crate::data::{load_projects_with_paths, IdeasPaths};

pub fn cmd_summarize(paths: &IdeasPaths, project: &str) -> Result<()> {
    let home = dirs::home_dir().expect("No home directory");
    let script = home.join("dotfiles/scripts/ideas/mq/generate-summary.sh");

    if project == "all-missing" {
        // Find all projects without summaries
        let projects = load_projects_with_paths(paths)?;
        let missing: Vec<_> = projects
            .iter()
            .filter(|p| p.summary.is_empty())
            .collect();

        if missing.is_empty() {
            println!("{}", "All projects have summaries!".green());
            return Ok(());
        }

        println!(
            "{}",
            format!("Generating summaries for {} projects...", missing.len()).cyan()
        );

        for p in &missing {
            println!("\n{}", format!("=== {} ===", p.name).yellow());

            let status = std::process::Command::new("bash")
                .arg(&script)
                .arg(&p.name)
                .status()?;

            if !status.success() {
                eprintln!("{}", format!("Failed to generate summary for {}", p.name).red());
            }
        }

        println!(
            "\n{}",
            "Done! Run 'icli refresh' to update the inventory.".green()
        );
    } else {
        // Single project
        let projects = load_projects_with_paths(paths)?;
        let found = projects.iter().find(|p| p.name == project);

        if found.is_none() {
            eprintln!("{}", format!("Project not found: {}", project).red());
            return Ok(());
        }

        let status = std::process::Command::new("bash")
            .arg(&script)
            .arg(project)
            .status()?;

        if status.success() {
            println!(
                "\n{}",
                "Done! Run 'icli refresh' to update the inventory.".green()
            );
        } else {
            eprintln!("{}", "Failed to generate summary".red());
        }
    }

    Ok(())
}
