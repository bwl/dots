//! Git command helpers.

use std::process::Command;

/// Get current HEAD commit (short hash) for a project.
pub fn get_project_head_commit(project_path: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", project_path, "rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Count commits since a given commit.
pub fn count_commits_since(project_path: &str, since_commit: &str) -> Option<u32> {
    let output = Command::new("git")
        .args([
            "-C",
            project_path,
            "rev-list",
            "--count",
            &format!("{}..HEAD", since_commit),
        ])
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .ok()
    } else {
        None
    }
}

