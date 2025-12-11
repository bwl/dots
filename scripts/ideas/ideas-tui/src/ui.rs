use crate::app::{App, ProjectDetailTab, Tab, View};
use crate::data::{has_analysis_file, SearchSource};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    match app.view {
        View::List => draw_list_view(frame, app),
        View::Detail => draw_detail_view(frame, app),
        View::MarkdownReader => draw_markdown_reader(frame, app),
        View::ProjectDetail => draw_project_detail(frame, app),
        View::PlanViewer => draw_plan_viewer(frame, app),
        View::GlobalSearch => draw_global_search(frame, app),
    }
}

fn draw_list_view(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs + header
            Constraint::Min(5),    // List
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    // Tabs header with all 5 tabs
    let tab_titles: Vec<String> = Tab::all()
        .iter()
        .map(|t| match t {
            Tab::Ideas => format!(" Ideas ({}) ", app.ideas.len()),
            Tab::Projects => format!(" Projects ({}) ", app.projects.len()),
            Tab::Plans => format!(" Plans ({}) ", app.plans.len()),
            Tab::Dotfiles => format!(" Dotfiles ({}) ", app.dotfiles.len()),
            Tab::Status => " Status ".to_string(),
        })
        .collect();

    let selected_tab = match app.tab {
        Tab::Ideas => 0,
        Tab::Projects => 1,
        Tab::Plans => 2,
        Tab::Dotfiles => 3,
        Tab::Status => 4,
    };

    let header_text = if app.search_mode {
        Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.search_query),
            Span::styled(
                "_",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ])
    } else {
        match app.tab {
            Tab::Ideas => Line::from(vec![
                Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
                Span::raw("Switch  "),
                Span::styled("[s] ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("Sort:{}  ", app.sort_by.label())),
                Span::styled("[f] ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("Filter:{}  ", app.filter.label())),
                Span::styled("[/] ", Style::default().fg(Color::Cyan)),
                Span::raw("Search  "),
                Span::styled("[^F] ", Style::default().fg(Color::Cyan)),
                Span::raw("Global"),
            ]),
            Tab::Projects => Line::from(vec![
                Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
                Span::raw("Switch  "),
                Span::styled("[s] ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("Sort:{}  ", app.project_sort_by.label())),
                Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
                Span::raw("Detail  "),
                Span::styled("[/] ", Style::default().fg(Color::Cyan)),
                Span::raw("Search"),
            ]),
            Tab::Plans => Line::from(vec![
                Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
                Span::raw("Switch  "),
                Span::styled("[s] ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("Sort:{}  ", app.plan_sort_by.label())),
                Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
                Span::raw("View  "),
                Span::styled("[/] ", Style::default().fg(Color::Cyan)),
                Span::raw("Search"),
            ]),
            Tab::Dotfiles => Line::from(vec![
                Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
                Span::raw("Switch  "),
                Span::styled("[s] ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("Sort:{}  ", app.dotfiles_sort_by.label())),
                Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
                Span::raw("Open  "),
                Span::styled("[/] ", Style::default().fg(Color::Cyan)),
                Span::raw("Search"),
            ]),
            Tab::Status => Line::from(vec![
                Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
                Span::raw("Switch  "),
                Span::styled("[r] ", Style::default().fg(Color::Cyan)),
                Span::raw("Refresh  "),
                Span::styled("[↑↓] ", Style::default().fg(Color::Cyan)),
                Span::raw("Sections"),
            ]),
        }
    };

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        .select(selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    // Split header area for tabs and controls
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(60), Constraint::Min(20)])
        .split(chunks[0]);

    frame.render_widget(tabs, header_chunks[0]);
    frame.render_widget(
        Paragraph::new(header_text).block(Block::default().borders(Borders::ALL)),
        header_chunks[1],
    );

    // List - different content based on tab
    match app.tab {
        Tab::Ideas => draw_ideas_list(frame, app, chunks[1]),
        Tab::Projects => draw_projects_list(frame, app, chunks[1]),
        Tab::Plans => draw_plans_list(frame, app, chunks[1]),
        Tab::Dotfiles => draw_dotfiles_list(frame, app, chunks[1]),
        Tab::Status => draw_status_dashboard(frame, app, chunks[1]),
    }

    // Status bar
    let status_bar = if app.search_mode {
        let matches = match app.tab {
            Tab::Ideas => app.filtered_indices.len(),
            Tab::Projects => app.project_filtered_indices.len(),
            Tab::Plans => app.plan_filtered_indices.len(),
            Tab::Dotfiles => app.dotfiles_filtered_indices.len(),
            Tab::Status => 0,
        };
        Paragraph::new(Line::from(vec![
            Span::styled(" [Esc] ", Style::default().fg(Color::Cyan)),
            Span::raw("Cancel  "),
            Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
            Span::raw("Accept  "),
            Span::styled(
                format!("{} matches", matches),
                Style::default().fg(Color::Yellow),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL))
    } else {
        match app.tab {
            Tab::Ideas => {
                let (total, active, dormant, questions) = app.idea_stats();
                Paragraph::new(Line::from(vec![
                    Span::raw(format!(" {} ideas", total)),
                    Span::raw(" │ "),
                    Span::styled(format!("{} active", active), Style::default().fg(Color::Green)),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{} dormant", dormant),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{} open questions", questions),
                        Style::default().fg(Color::Cyan),
                    ),
                ]))
                .block(Block::default().borders(Borders::ALL))
            }
            Tab::Projects => {
                let (total, analyzed) = app.project_stats();
                Paragraph::new(Line::from(vec![
                    Span::raw(format!(" {} projects", total)),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{} analyzed", analyzed),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{} pending", total - analyzed),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
                .block(Block::default().borders(Borders::ALL))
            }
            Tab::Plans => {
                let total = app.plan_stats();
                Paragraph::new(Line::from(vec![Span::raw(format!(" {} plans", total))]))
                    .block(Block::default().borders(Borders::ALL))
            }
            Tab::Dotfiles => {
                let total = app.dotfiles_stats();
                Paragraph::new(Line::from(vec![Span::raw(format!(" {} items", total))]))
                    .block(Block::default().borders(Borders::ALL))
            }
            Tab::Status => {
                let (untracked, stale, recent) = app.status_stats();
                Paragraph::new(Line::from(vec![
                    Span::styled(
                        format!(" {} new", untracked),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(" │ "),
                    Span::styled(format!("{} stale", stale), Style::default().fg(Color::Yellow)),
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{} recent (7d)", recent),
                        Style::default().fg(Color::Cyan),
                    ),
                ]))
                .block(Block::default().borders(Borders::ALL))
            }
        }
    };
    frame.render_widget(status_bar, chunks[2]);
}

fn draw_ideas_list(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let ideas = app.filtered_ideas();
    // Calculate available width for tags: total - borders - fixed columns
    // Fixed: name (20) + status (10) + questions (4) + sessions (3) + highlight (2) = 39
    let tags_width = (area.width as usize).saturating_sub(43).max(10);

    let items: Vec<ListItem> = ideas
        .iter()
        .map(|idea| {
            let status_color = match idea.status.as_str() {
                "active" => Color::Green,
                "dormant" => Color::Yellow,
                "unknown" => Color::Red,
                _ => Color::Gray,
            };

            let tags_str = if idea.tags.is_empty() {
                String::new()
            } else {
                idea.tags.join(", ")
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<20}", truncate(&idea.folder, 19)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<10}", truncate(&idea.status, 9)),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    format!("{:<width$}", truncate(&tags_str, tags_width - 1), width = tags_width),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(
                    format!("{:>2}q ", idea.open_questions.len()),
                    Style::default().fg(if idea.open_questions.is_empty() {
                        Color::DarkGray
                    } else {
                        Color::Cyan
                    }),
                ),
                Span::styled(
                    format!("{:>2}s", idea.sessions),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_projects_list(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let projects = app.filtered_projects();
    // Calculate available width for description: total - borders - fixed columns
    // Fixed: [A] (3) + space (1) + name (18) + category (12) + date (12) + highlight (2) = 48
    let desc_width = (area.width as usize).saturating_sub(50);

    let items: Vec<ListItem> = projects
        .iter()
        .map(|p| {
            let analyzed = has_analysis_file(&p.name);
            let analysis_indicator = if analyzed {
                Span::styled("[A]", Style::default().fg(Color::Green))
            } else {
                Span::styled("[ ]", Style::default().fg(Color::DarkGray))
            };

            let cat_color = match p.category.as_str() {
                "roguelike" => Color::Red,
                "writing" => Color::Magenta,
                "knowledge" => Color::Blue,
                "simulation" => Color::Green,
                "tui" | "cli" => Color::Cyan,
                _ => Color::White,
            };

            let line = Line::from(vec![
                analysis_indicator,
                Span::raw(" "),
                Span::styled(
                    format!("{:<18}", truncate(&p.name, 17)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<12}", truncate(&p.category, 11)),
                    Style::default().fg(cat_color),
                ),
                Span::styled(
                    format!("{:<12}", &p.last_commit),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(truncate(&p.description, desc_width)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.project_list_state);
}

fn draw_plans_list(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let plans = app.filtered_plans();
    // Calculate available width for title: total - borders - fixed columns
    // Fixed: name (30) + date (10) + highlight (2) = 42
    let title_width = (area.width as usize).saturating_sub(46).max(20);

    let items: Vec<ListItem> = plans
        .iter()
        .map(|plan| {
            let line = Line::from(vec![
                Span::styled(
                    format!("{:<30}", truncate(&plan.name, 29)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<width$}", truncate(&plan.title, title_width - 1), width = title_width),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("{}", &plan.modified),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.plan_list_state);
}

fn draw_dotfiles_list(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let dotfiles = app.filtered_dotfiles();
    // Calculate available width for description: total - borders - fixed columns
    // Fixed: name (20) + category (12) + path (30) + highlight (2) = 64
    let desc_width = (area.width as usize).saturating_sub(68).max(10);

    let items: Vec<ListItem> = dotfiles
        .iter()
        .map(|item| {
            let cat_color = match item.category.as_str() {
                "dx-script" => Color::Green,
                "app-config" => Color::Blue,
                "shell" => Color::Yellow,
                "editor" => Color::Magenta,
                _ => Color::White,
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<20}", truncate(&item.name, 19)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<12}", truncate(&item.category, 11)),
                    Style::default().fg(cat_color),
                ),
                Span::styled(
                    format!("{:<30}", truncate(&item.path, 29)),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(truncate(&item.description, desc_width)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.dotfiles_list_state);
}

fn draw_status_dashboard(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 3), // Untracked
            Constraint::Ratio(1, 3), // Stale
            Constraint::Ratio(1, 3), // Recent
        ])
        .split(area);

    // Untracked projects section
    let untracked_items: Vec<ListItem> = app
        .untracked_projects
        .iter()
        .take(5)
        .map(|p| {
            let line = Line::from(vec![
                Span::styled(
                    format!("{:<20}", truncate(&p.name, 19)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<15}", truncate(&p.tech, 14)),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("{} commits", p.commits),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let untracked_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " Untracked Projects ({}) ",
            app.untracked_projects.len()
        ))
        .border_style(if app.status_section == 0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let untracked_list = List::new(untracked_items).block(untracked_block);
    frame.render_widget(untracked_list, chunks[0]);

    // Stale analyses section
    let stale_items: Vec<ListItem> = app
        .stale_projects
        .iter()
        .take(5)
        .map(|(name, commits)| {
            let line = Line::from(vec![
                Span::styled(
                    format!("{:<25}", truncate(name, 24)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{} commits behind", commits),
                    Style::default().fg(Color::Yellow),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let stale_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Stale Analyses ({}) ", app.stale_projects.len()))
        .border_style(if app.status_section == 1 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let stale_list = List::new(stale_items).block(stale_block);
    frame.render_widget(stale_list, chunks[1]);

    // Recent activity section
    let recent_items: Vec<ListItem> = app
        .recent_activity
        .iter()
        .take(5)
        .map(|p| {
            let line = Line::from(vec![
                Span::styled(
                    format!("{:<20}", truncate(&p.name, 19)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<12}", &p.last_commit_date),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(truncate(&p.last_commit_msg, 40)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let recent_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Recent Activity ({}) ", app.recent_activity.len()))
        .border_style(if app.status_section == 2 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let recent_list = List::new(recent_items).block(recent_block);
    frame.render_widget(recent_list, chunks[2]);
}

fn draw_detail_view(frame: &mut Frame, app: &mut App) {
    let idea = match app.selected_idea() {
        Some(i) => i.clone(),
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Info
            Constraint::Min(3),     // Questions
            Constraint::Length(8),  // Markdown files
            Constraint::Length(3),  // Actions
        ])
        .split(frame.area());

    // Info section
    let status_color = match idea.status.as_str() {
        "active" => Color::Green,
        "dormant" => Color::Yellow,
        _ => Color::Red,
    };

    let info_lines = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&idea.status, Style::default().fg(status_color)),
            Span::raw("    "),
            Span::styled("Sessions: ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{}", idea.sessions)),
        ]),
        Line::from(vec![
            Span::styled("Tags: ", Style::default().fg(Color::DarkGray)),
            Span::raw(idea.tags.join(", ")),
        ]),
        Line::from(""),
        Line::from(Span::raw(&idea.description)),
    ];

    let info = Paragraph::new(info_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", idea.folder)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(info, chunks[0]);

    // Questions section
    let questions: Vec<Line> = if idea.open_questions.is_empty() {
        vec![Line::from(Span::styled(
            "No open questions",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        idea.open_questions
            .iter()
            .map(|q| {
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::raw(q),
                ])
            })
            .collect()
    };

    let questions_widget = Paragraph::new(questions)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Open Questions ({}) ", idea.open_questions.len())),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(questions_widget, chunks[1]);

    // Markdown files section
    let md_items: Vec<ListItem> = app
        .md_files
        .iter()
        .map(|path| {
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            ListItem::new(Line::from(Span::raw(format!("  {}", filename))))
        })
        .collect();

    let md_list = List::new(md_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Markdown Files ({}) ", app.md_files.len())),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(md_list, chunks[2], &mut app.md_list_state);

    // Actions
    let actions = Paragraph::new(Line::from(vec![
        Span::styled(" [↑↓/jk] ", Style::default().fg(Color::Cyan)),
        Span::raw("Select  "),
        Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
        Span::raw("Read  "),
        Span::styled("[e] ", Style::default().fg(Color::Cyan)),
        Span::raw("$EDITOR  "),
        Span::styled("[o] ", Style::default().fg(Color::Cyan)),
        Span::raw("Folder  "),
        Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
        Span::raw("Back"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(actions, chunks[3]);
}

fn draw_project_detail(frame: &mut Frame, app: &mut App) {
    let project = match app.selected_project() {
        Some(p) => p.clone(),
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(5),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    // Tab bar
    let tab_titles = vec![" Info ", " Analysis "];
    let selected_tab = match app.project_detail_tab {
        ProjectDetailTab::Info => 0,
        ProjectDetailTab::Analysis => 1,
    };

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", project.name)),
        )
        .select(selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, chunks[0]);

    // Content based on selected tab
    match app.project_detail_tab {
        ProjectDetailTab::Info => {
            let analyzed = has_analysis_file(&project.name);
            let analysis_status = if analyzed {
                Span::styled("Yes", Style::default().fg(Color::Green))
            } else {
                Span::styled("No", Style::default().fg(Color::DarkGray))
            };

            let info_lines = vec![
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(&project.path),
                ]),
                Line::from(vec![
                    Span::styled("Category: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(&project.category),
                ]),
                Line::from(vec![
                    Span::styled("Tech: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(&project.tech),
                ]),
                Line::from(vec![
                    Span::styled("Total commits: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{}", project.commits)),
                ]),
                Line::from(vec![
                    Span::styled("Last commit: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(&project.last_commit),
                ]),
                Line::from(vec![
                    Span::styled("Analyzed: ", Style::default().fg(Color::DarkGray)),
                    analysis_status,
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(Span::raw(&project.description)),
            ];

            let info = Paragraph::new(info_lines)
                .block(Block::default().borders(Borders::ALL))
                .scroll((app.project_info_scroll, 0))
                .wrap(Wrap { trim: true });
            frame.render_widget(info, chunks[1]);
        }
        ProjectDetailTab::Analysis => {
            if app.analysis_content.is_empty() {
                let no_analysis = Paragraph::new(Line::from(vec![
                    Span::styled(
                        "No analysis available. Run: ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("icli analyze {}", project.name),
                        Style::default().fg(Color::Cyan),
                    ),
                ]))
                .block(Block::default().borders(Borders::ALL));
                frame.render_widget(no_analysis, chunks[1]);
            } else {
                let text = tui_markdown::from_str(&app.analysis_content);
                let paragraph = Paragraph::new(text)
                    .scroll((app.analysis_scroll, 0))
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: false });
                frame.render_widget(paragraph, chunks[1]);
            }
        }
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [Tab] ", Style::default().fg(Color::Cyan)),
        Span::raw("Switch tab  "),
        Span::styled("[↑↓/jk] ", Style::default().fg(Color::Cyan)),
        Span::raw("Scroll  "),
        Span::styled("[o] ", Style::default().fg(Color::Cyan)),
        Span::raw("Open Folder  "),
        Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
        Span::raw("Back"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

fn draw_plan_viewer(frame: &mut Frame, app: &App) {
    let plan_name = app
        .selected_plan()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    // Render plan content as markdown
    let text = tui_markdown::from_str(&app.plan_content);
    let paragraph = Paragraph::new(text)
        .scroll((app.plan_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", plan_name)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, chunks[0]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [↑↓/jk] ", Style::default().fg(Color::Cyan)),
        Span::raw("Scroll  "),
        Span::styled("[d/u] ", Style::default().fg(Color::Cyan)),
        Span::raw("Page  "),
        Span::styled("[g/G] ", Style::default().fg(Color::Cyan)),
        Span::raw("Top/Bottom  "),
        Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
        Span::raw("Back  "),
        Span::styled(
            format!("Line {}/{}", app.plan_scroll + 1, app.plan_total_lines),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[1]);
}

fn draw_global_search(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(5),    // Results
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    // Search bar
    let search_text = if app.search_mode {
        Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.search_query),
            Span::styled(
                "_",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(Color::DarkGray)),
            Span::raw(&app.search_query),
        ])
    };

    let search_bar = Paragraph::new(search_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Global Search "),
        );
    frame.render_widget(search_bar, chunks[0]);

    // Results
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .map(|result| {
            let source_color = match result.source {
                SearchSource::Ideas => Color::Green,
                SearchSource::Projects => Color::Blue,
                SearchSource::Plans => Color::Magenta,
                SearchSource::Dotfiles => Color::Cyan,
            };

            let source_label = match result.source {
                SearchSource::Ideas => "[Idea]",
                SearchSource::Projects => "[Proj]",
                SearchSource::Plans => "[Plan]",
                SearchSource::Dotfiles => "[Dots]",
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<7}", source_label),
                    Style::default().fg(source_color),
                ),
                Span::styled(
                    format!("{:<25}", truncate(&result.name, 24)),
                    Style::default().fg(Color::White),
                ),
                Span::raw(truncate(&result.description, 40)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let results_title = format!(" Results ({}) ", app.search_results.len());
    let results_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(results_title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(results_list, chunks[1], &mut app.search_results_state);

    // Footer
    let footer = if app.search_mode {
        Paragraph::new(Line::from(vec![
            Span::styled(" [Esc] ", Style::default().fg(Color::Cyan)),
            Span::raw("Cancel  "),
            Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
            Span::raw("Confirm"),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(" [↑↓/jk] ", Style::default().fg(Color::Cyan)),
            Span::raw("Select  "),
            Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
            Span::raw("Jump to  "),
            Span::styled("[/] ", Style::default().fg(Color::Cyan)),
            Span::raw("New search  "),
            Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
            Span::raw("Back"),
        ]))
    };

    frame.render_widget(
        footer.block(Block::default().borders(Borders::ALL)),
        chunks[2],
    );
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn draw_markdown_reader(frame: &mut Frame, app: &App) {
    let filename = app
        .selected_md_file()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    // Render markdown content
    let text = tui_markdown::from_str(&app.md_content);
    let paragraph = Paragraph::new(text)
        .scroll((app.md_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", filename)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, chunks[0]);

    // Footer with scroll position
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [↑↓/jk] ", Style::default().fg(Color::Cyan)),
        Span::raw("Scroll  "),
        Span::styled("[d/u] ", Style::default().fg(Color::Cyan)),
        Span::raw("Page  "),
        Span::styled("[g/G] ", Style::default().fg(Color::Cyan)),
        Span::raw("Top/Bottom  "),
        Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
        Span::raw("Back  "),
        Span::styled(
            format!("Line {}/{}", app.md_scroll + 1, app.md_total_lines),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[1]);
}

