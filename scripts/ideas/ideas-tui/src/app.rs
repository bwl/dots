use crate::data::{find_markdown_files, sort_ideas, Idea, SortBy};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::widgets::ListState;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    List,
    Detail,
    MarkdownReader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusFilter {
    All,
    Active,
    Dormant,
    Unknown,
}

impl StatusFilter {
    pub fn next(self) -> Self {
        match self {
            StatusFilter::All => StatusFilter::Active,
            StatusFilter::Active => StatusFilter::Dormant,
            StatusFilter::Dormant => StatusFilter::Unknown,
            StatusFilter::Unknown => StatusFilter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            StatusFilter::All => "all",
            StatusFilter::Active => "active",
            StatusFilter::Dormant => "dormant",
            StatusFilter::Unknown => "unknown",
        }
    }

    pub fn matches(self, status: &str) -> bool {
        match self {
            StatusFilter::All => true,
            StatusFilter::Active => status == "active",
            StatusFilter::Dormant => status == "dormant",
            StatusFilter::Unknown => status == "unknown",
        }
    }
}

pub struct App {
    pub ideas: Vec<Idea>,
    pub filtered_indices: Vec<usize>,
    pub list_state: ListState,
    pub view: View,
    pub sort_by: SortBy,
    pub filter: StatusFilter,
    pub repo_root: PathBuf,
    pub should_quit: bool,
    // Markdown viewer state
    pub md_files: Vec<PathBuf>,
    pub md_list_state: ListState,
    pub md_content: String,
    pub md_scroll: u16,
    pub md_total_lines: u16,
    // Search state
    pub search_mode: bool,
    pub search_query: String,
}

impl App {
    pub fn new(ideas: Vec<Idea>, repo_root: PathBuf) -> Self {
        let filtered_indices: Vec<usize> = (0..ideas.len()).collect();
        let mut list_state = ListState::default();
        if !filtered_indices.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            ideas,
            filtered_indices,
            list_state,
            view: View::List,
            sort_by: SortBy::Name,
            filter: StatusFilter::All,
            repo_root,
            should_quit: false,
            md_files: Vec::new(),
            md_list_state: ListState::default(),
            md_content: String::new(),
            md_scroll: 0,
            md_total_lines: 0,
            search_mode: false,
            search_query: String::new(),
        }
    }

    pub fn filtered_ideas(&self) -> Vec<&Idea> {
        self.filtered_indices
            .iter()
            .map(|&i| &self.ideas[i])
            .collect()
    }

    pub fn selected_idea(&self) -> Option<&Idea> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i))
            .map(|&idx| &self.ideas[idx])
    }

    fn matches_search(&self, idea: &Idea) -> bool {
        if self.search_query.is_empty() {
            return true;
        }
        let query = self.search_query.to_lowercase();
        idea.folder.to_lowercase().contains(&query)
            || idea.description.to_lowercase().contains(&query)
            || idea.tags.iter().any(|t| t.to_lowercase().contains(&query))
    }

    pub fn update_filter(&mut self) {
        self.filtered_indices = self
            .ideas
            .iter()
            .enumerate()
            .filter(|(_, idea)| self.filter.matches(&idea.status) && self.matches_search(idea))
            .map(|(i, _)| i)
            .collect();

        // Reset selection
        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_by = self.sort_by.next();
        sort_ideas(&mut self.ideas, self.sort_by);
        self.update_filter();
    }

    pub fn cycle_filter(&mut self) {
        self.filter = self.filter.next();
        self.update_filter();
    }

    pub fn next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_indices.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_indices.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn enter_detail(&mut self) {
        if let Some(idea) = self.selected_idea() {
            let idea_path = self.repo_root.join(&idea.folder);
            self.md_files = find_markdown_files(&idea_path);
            self.md_list_state = ListState::default();
            if !self.md_files.is_empty() {
                self.md_list_state.select(Some(0));
            }
            self.view = View::Detail;
        }
    }

    pub fn back_to_list(&mut self) {
        self.view = View::List;
    }

    pub fn open_in_editor(&self) {
        if let Some(idea) = self.selected_idea() {
            let readme_path = self.repo_root.join(&idea.folder).join("README.md");
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
            let _ = Command::new(&editor).arg(&readme_path).status();
        }
    }

    pub fn open_folder(&self) {
        if let Some(idea) = self.selected_idea() {
            let folder_path = self.repo_root.join(&idea.folder);
            // macOS open command
            let _ = Command::new("open").arg(&folder_path).status();
        }
    }

    pub fn md_next(&mut self) {
        if self.md_files.is_empty() {
            return;
        }
        let i = match self.md_list_state.selected() {
            Some(i) => {
                if i >= self.md_files.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.md_list_state.select(Some(i));
    }

    pub fn md_previous(&mut self) {
        if self.md_files.is_empty() {
            return;
        }
        let i = match self.md_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.md_files.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.md_list_state.select(Some(i));
    }

    pub fn open_markdown_file(&mut self) {
        if let Some(idx) = self.md_list_state.selected() {
            if let Some(path) = self.md_files.get(idx) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    self.md_total_lines = content.lines().count() as u16;
                    self.md_content = content;
                    self.md_scroll = 0;
                    self.view = View::MarkdownReader;
                }
            }
        }
    }

    pub fn selected_md_file(&self) -> Option<&PathBuf> {
        self.md_list_state
            .selected()
            .and_then(|i| self.md_files.get(i))
    }

    pub fn md_scroll_down(&mut self, amount: u16) {
        self.md_scroll = self.md_scroll.saturating_add(amount);
    }

    pub fn md_scroll_up(&mut self, amount: u16) {
        self.md_scroll = self.md_scroll.saturating_sub(amount);
    }

    pub fn md_scroll_to_top(&mut self) {
        self.md_scroll = 0;
    }

    pub fn md_scroll_to_bottom(&mut self) {
        self.md_scroll = self.md_total_lines.saturating_sub(10);
    }

    pub fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) {
        match self.view {
            View::List => {
                if self.search_mode {
                    match code {
                        KeyCode::Esc => {
                            self.search_mode = false;
                            self.search_query.clear();
                            self.update_filter();
                        }
                        KeyCode::Enter => {
                            self.search_mode = false;
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                            self.update_filter();
                        }
                        KeyCode::Char(c) => {
                            self.search_query.push(c);
                            self.update_filter();
                        }
                        _ => {}
                    }
                } else {
                    match code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('j') | KeyCode::Down => self.next(),
                        KeyCode::Char('k') | KeyCode::Up => self.previous(),
                        KeyCode::Enter => self.enter_detail(),
                        KeyCode::Char('s') => self.cycle_sort(),
                        KeyCode::Char('f') => self.cycle_filter(),
                        KeyCode::Char('e') => self.open_in_editor(),
                        KeyCode::Char('o') => self.open_folder(),
                        KeyCode::Char('/') => {
                            self.search_mode = true;
                            self.search_query.clear();
                        }
                        _ => {}
                    }
                }
            }
            View::Detail => match code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Esc => self.back_to_list(),
                KeyCode::Char('j') | KeyCode::Down => self.md_next(),
                KeyCode::Char('k') | KeyCode::Up => self.md_previous(),
                KeyCode::Enter => self.open_markdown_file(),
                KeyCode::Char('e') => self.open_in_editor(),
                KeyCode::Char('o') => self.open_folder(),
                _ => {}
            },
            View::MarkdownReader => match code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Esc => self.view = View::Detail,
                KeyCode::Char('j') | KeyCode::Down => self.md_scroll_down(1),
                KeyCode::Char('k') | KeyCode::Up => self.md_scroll_up(1),
                KeyCode::Char('d') => self.md_scroll_down(10),
                KeyCode::Char('u') => self.md_scroll_up(10),
                KeyCode::Char('g') => self.md_scroll_to_top(),
                KeyCode::Char('G') => self.md_scroll_to_bottom(),
                KeyCode::PageDown => self.md_scroll_down(20),
                KeyCode::PageUp => self.md_scroll_up(20),
                _ => {}
            },
        }
    }

    // Stats for status bar
    pub fn stats(&self) -> (usize, usize, usize, usize) {
        let total = self.ideas.len();
        let active = self.ideas.iter().filter(|i| i.status == "active").count();
        let dormant = self.ideas.iter().filter(|i| i.status == "dormant").count();
        let questions: usize = self.ideas.iter().map(|i| i.open_questions.len()).sum();
        (total, active, dormant, questions)
    }
}
