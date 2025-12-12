use anyhow::Result;
use colored::Colorize;

use crate::data::{analysis_dir_with_paths, chrono_lite, IdeasPaths, Project};

pub fn cmd_snapshot(paths: &IdeasPaths, output: Option<String>) -> Result<()> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    let home = dirs::home_dir().expect("No home directory");
    let ideas_root = &paths.ideas_repo;

    let today = chrono_lite(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    );
    let default_name = format!("ideas-snapshot-{}.zip", today);
    let output_path = match output {
        Some(p) => std::path::PathBuf::from(p),
        None => home.join("Downloads").join(&default_name),
    };

    println!("{}", "Creating NotebookLM snapshot...".cyan());

    let file = std::fs::File::create(&output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. Analysis files (already markdown)
    let analysis_dir = analysis_dir_with_paths(paths);
    let mut analysis_count = 0;
    if analysis_dir.exists() {
        for entry in std::fs::read_dir(&analysis_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") {
                let name =
                    format!("analysis/{}", path.file_name().unwrap().to_string_lossy());
                let content = std::fs::read(&path)?;
                zip.start_file(&name, options)?;
                zip.write_all(&content)?;
                analysis_count += 1;
            }
        }
    }

    // 2. Project inventory (JSON → Markdown)
    let inventory_path = &paths.project_inventory_path;
    if inventory_path.exists() {
        let md = inventory_to_markdown(inventory_path)?;
        zip.start_file("project-inventory.md", options)?;
        zip.write_all(md.as_bytes())?;
    }

    // 3. Tracker (CSV → Markdown)
    let tracker_path = ideas_root.join("_tracker.csv");
    if tracker_path.exists() {
        let md = tracker_to_markdown(&tracker_path)?;
        zip.start_file("ideas-tracker.md", options)?;
        zip.write_all(md.as_bytes())?;
    }

    // 4. Repo docs (already markdown)
    for name in ["README.md", "CLAUDE.md"] {
        let path = ideas_root.join(name);
        if path.exists() {
            let content = std::fs::read(&path)?;
            zip.start_file(name, options)?;
            zip.write_all(&content)?;
        }
    }

    zip.finish()?;

    let zip_size = std::fs::metadata(&output_path)?.len();
    println!("  {} analysis files", analysis_count.to_string().bold());
    println!("  {} project inventory (as markdown)", "1".bold());
    println!("  {} tracker (as markdown)", "1".bold());
    println!("  {} repo docs", "2".bold());
    println!();
    println!(
        "{}: {} ({} compressed)",
        "Snapshot".green(),
        output_path.display(),
        format_size(zip_size)
    );

    Ok(())
}

fn inventory_to_markdown(path: &std::path::Path) -> Result<String> {
    #[derive(serde::Deserialize)]
    struct Inventory {
        projects: Vec<Project>,
    }

    let content = std::fs::read_to_string(path)?;
    let inv: Inventory = serde_json::from_str(&content)?;

    let mut md = String::from("# Project Inventory\n\n");
    md.push_str(&format!("Total projects: {}\n\n", inv.projects.len()));

    let mut by_cat: std::collections::HashMap<&str, Vec<&Project>> =
        std::collections::HashMap::new();
    for p in &inv.projects {
        by_cat.entry(&p.category).or_default().push(p);
    }

    let mut cats: Vec<_> = by_cat.keys().collect();
    cats.sort();

    for cat in cats {
        let projects = &by_cat[cat];
        md.push_str(&format!("## {} ({} projects)\n\n", cat, projects.len()));

        for p in projects {
            let desc = if !p.summary.is_empty() { &p.summary } else { &p.description };
            md.push_str(&format!("### {}\n\n", p.name));
            md.push_str(&format!("- **Path**: {}\n", p.path));
            md.push_str(&format!("- **Tech**: {}\n", p.tech));
            md.push_str(&format!("- **Last commit**: {}\n", p.last_commit));
            md.push_str(&format!("- **Description**: {}\n\n", desc));
        }
    }

    Ok(md)
}

fn tracker_to_markdown(path: &std::path::Path) -> Result<String> {
    let mut md = String::from("# Ideas Tracker\n\n");
    md.push_str("| Folder | Tags | Description | Created | Modified | Sessions |\n");
    md.push_str("|--------|------|-------------|---------|----------|----------|\n");

    let mut reader = csv::Reader::from_path(path)?;
    for result in reader.records() {
        let record = result?;
        let folder = record.get(0).unwrap_or("");
        let tags = record.get(1).unwrap_or("").trim_matches('"');
        let description = record.get(2).unwrap_or("").trim_matches('"');
        let created = record.get(3).unwrap_or("");
        let modified = record.get(4).unwrap_or("");
        let sessions = record.get(5).unwrap_or("0");

        if !folder.is_empty() {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                folder, tags, description, created, modified, sessions
            ));
        }
    }

    Ok(md)
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
