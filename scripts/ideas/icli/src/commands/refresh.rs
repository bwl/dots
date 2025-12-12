use anyhow::Result;
use colored::Colorize;

use crate::data::IdeasPaths;

pub fn cmd_refresh(_paths: &IdeasPaths) -> Result<()> {
    println!("Refreshing project inventory...");

    let home = dirs::home_dir().expect("No home directory");
    let scan_script = home.join("dotfiles/scripts/ideas/mq/projects-scan.sh");

    let status = std::process::Command::new("bash")
        .arg(&scan_script)
        .status()?;

    if status.success() {
        println!("{}", "Done!".green());
    } else {
        println!("{}", "Failed to refresh inventory".red());
    }

    Ok(())
}

