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
use data::{
    find_ideas_repo_with_paths, load_dotfiles_with_paths, load_ideas, load_plans_with_paths,
    load_projects_with_paths, IdeasPaths,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

fn main() -> Result<()> {
    let paths = IdeasPaths::detect()?;

    // Find repo root (look for _tracker.csv)
    let repo_root = find_ideas_repo_with_paths(&paths)?;

    // Load all data sources
    let ideas = load_ideas(&repo_root)?;
    let projects = load_projects_with_paths(&paths).unwrap_or_default();
    let plans = load_plans_with_paths(&paths).unwrap_or_default();
    let dotfiles = load_dotfiles_with_paths(&paths).unwrap_or_default();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with all data sources
    let mut app = App::new(paths, ideas, projects, plans, dotfiles, repo_root);

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
        app.tick();
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code, key.modifiers);
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
