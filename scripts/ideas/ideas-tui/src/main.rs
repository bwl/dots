mod app;
mod data;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use data::{load_ideas, load_projects};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Find repo root (look for _tracker.csv)
    let repo_root = find_repo_root()?;

    // Load ideas and projects
    let ideas = load_ideas(&repo_root)?;
    let projects = load_projects().unwrap_or_default();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(ideas, projects, repo_root);

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code, key.modifiers);
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn find_repo_root() -> Result<PathBuf> {
    // Try current directory first
    let cwd = std::env::current_dir()?;
    if cwd.join("_tracker.csv").exists() {
        return Ok(cwd);
    }

    // Try to find it via git
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let path = PathBuf::from(path);
        if path.join("_tracker.csv").exists() {
            return Ok(path);
        }
    }

    anyhow::bail!("Could not find ideas repo (no _tracker.csv found)")
}
