use crate::app::{App, Tab, View};
use crate::data::has_analysis_file;
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
        View::AnalysisPreview => draw_analysis_preview(frame, app),
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

    // Tabs header
    let tab_titles = vec![
        format!(" Ideas ({}) ", app.ideas.len()),
        format!(" Projects ({}) ", app.projects.len()),
    ];
    let selected_tab = match app.tab {
        Tab::Ideas => 0,
        Tab::Projects => 1,
    };

    let header_text = if app.search_mode {
        Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.search_query),
            Span::styled("_", Style::default().fg(Color::Yellow).add_modifier(Modifier::SLOW_BLINK)),
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
                Span::raw("Search"),
            ]),
            Tab::Projects => Line::from(vec![
                Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
                Span::raw("Switch  "),
                Span::styled("[s] ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("Sort:{}  ", app.project_sort_by.label())),
                Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
                Span::raw("Analysis  "),
                Span::styled("[/] ", Style::default().fg(Color::Cyan)),
                Span::raw("Search"),
            ]),
        }
    };

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        .select(selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    // Split header area for tabs and controls
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(20)])
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
    }

    // Status bar
    let status_bar = if app.search_mode {
        let matches = match app.tab {
            Tab::Ideas => app.filtered_indices.len(),
            Tab::Projects => app.project_filtered_indices.len(),
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
                let (total, active, dormant, questions) = app.stats();
                Paragraph::new(Line::from(vec![
                    Span::raw(format!(" {} ideas", total)),
                    Span::raw(" │ "),
                    Span::styled(format!("{} active", active), Style::default().fg(Color::Green)),
                    Span::raw(" │ "),
                    Span::styled(format!("{} dormant", dormant), Style::default().fg(Color::Yellow)),
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
        }
    };
    frame.render_widget(status_bar, chunks[2]);
}

fn draw_ideas_list(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let ideas = app.filtered_ideas();
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
                    format!("{:<20}", truncate(&tags_str, 19)),
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
                Span::raw(truncate(&p.description, 30)),
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

fn draw_analysis_preview(frame: &mut Frame, app: &App) {
    let project_name = app
        .selected_project()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    // Render analysis content as markdown
    let text = tui_markdown::from_str(&app.analysis_content);
    let paragraph = Paragraph::new(text)
        .scroll((app.analysis_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Analysis: {} ", project_name)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, chunks[0]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [↑↓/jk] ", Style::default().fg(Color::Cyan)),
        Span::raw("Scroll  "),
        Span::styled("[d/u] ", Style::default().fg(Color::Cyan)),
        Span::raw("Page  "),
        Span::styled("[o] ", Style::default().fg(Color::Cyan)),
        Span::raw("Open Folder  "),
        Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
        Span::raw("Back  "),
        Span::styled(
            format!("Line {}/{}", app.analysis_scroll + 1, app.analysis_total_lines),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[1]);
}
