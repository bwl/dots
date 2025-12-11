use crate::data::{
    find_markdown_files, has_analysis_file, load_analysis_summary, sort_ideas, sort_projects,
    Idea, Project, ProjectSortBy, SortBy,
};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::widgets::ListState;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Ideas,
    Projects,
}

impl Tab {
    pub fn next(self) -> Self {
        match self {
            Tab::Ideas => Tab::Projects,
            Tab::Projects => Tab::Ideas,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    List,
    Detail,
    MarkdownReader,
    AnalysisPreview,
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
    // Tab state
    pub tab: Tab,
    // Ideas state
    pub ideas: Vec<Idea>,
    pub filtered_indices: Vec<usize>,
    pub list_state: ListState,
    pub view: View,
    pub sort_by: SortBy,
    pub filter: StatusFilter,
    pub repo_root: PathBuf,
    pub should_quit: bool,
    // Projects state
    pub projects: Vec<Project>,
    pub project_filtered_indices: Vec<usize>,
    pub project_list_state: ListState,
    pub project_sort_by: ProjectSortBy,
    pub project_category_filter: Option<String>,
    pub analysis_content: String,
    pub analysis_scroll: u16,
    pub analysis_total_lines: u16,
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
    pub fn new(ideas: Vec<Idea>, projects: Vec<Project>, repo_root: PathBuf) -> Self {
        let filtered_indices: Vec<usize> = (0..ideas.len()).collect();
        let mut list_state = ListState::default();
        if !filtered_indices.is_empty() {
            list_state.select(Some(0));
        }

        let project_filtered_indices: Vec<usize> = (0..projects.len()).collect();
        let mut project_list_state = ListState::default();
        if !project_filtered_indices.is_empty() {
            project_list_state.select(Some(0));
        }

        Self {
            tab: Tab::Ideas,
            ideas,
            filtered_indices,
            list_state,
            view: View::List,
            sort_by: SortBy::Name,
            filter: StatusFilter::All,
            repo_root,
            should_quit: false,
            projects,
            project_filtered_indices,
            project_list_state,
            project_sort_by: ProjectSortBy::Analyzed,
            project_category_filter: None,
            analysis_content: String::new(),
            analysis_scroll: 0,
            analysis_total_lines: 0,
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

    // ============ Project Methods ============

    pub fn filtered_projects(&self) -> Vec<&Project> {
        self.project_filtered_indices
            .iter()
            .map(|&i| &self.projects[i])
            .collect()
    }

    pub fn selected_project(&self) -> Option<&Project> {
        self.project_list_state
            .selected()
            .and_then(|i| self.project_filtered_indices.get(i))
            .map(|&idx| &self.projects[idx])
    }

    pub fn update_project_filter(&mut self) {
        self.project_filtered_indices = self
            .projects
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if let Some(ref cat) = self.project_category_filter {
                    if &p.category != cat {
                        return false;
                    }
                }
                if !self.search_query.is_empty() {
                    let q = self.search_query.to_lowercase();
                    if !p.name.to_lowercase().contains(&q)
                        && !p.description.to_lowercase().contains(&q)
                        && !p.category.to_lowercase().contains(&q)
                    {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();

        if self.project_filtered_indices.is_empty() {
            self.project_list_state.select(None);
        } else {
            self.project_list_state.select(Some(0));
        }
    }

    pub fn cycle_project_sort(&mut self) {
        self.project_sort_by = self.project_sort_by.next();
        sort_projects(&mut self.projects, self.project_sort_by);
        self.update_project_filter();
    }

    pub fn project_next(&mut self) {
        if self.project_filtered_indices.is_empty() {
            return;
        }
        let i = match self.project_list_state.selected() {
            Some(i) => {
                if i >= self.project_filtered_indices.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.project_list_state.select(Some(i));
    }

    pub fn project_previous(&mut self) {
        if self.project_filtered_indices.is_empty() {
            return;
        }
        let i = match self.project_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.project_filtered_indices.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.project_list_state.select(Some(i));
    }

    pub fn open_analysis_preview(&mut self) {
        if let Some(project) = self.selected_project() {
            if let Some(content) = load_analysis_summary(&project.name) {
                self.analysis_total_lines = content.lines().count() as u16;
                self.analysis_content = content;
                self.analysis_scroll = 0;
                self.view = View::AnalysisPreview;
            }
        }
    }

    pub fn open_project_folder(&self) {
        if let Some(project) = self.selected_project() {
            let _ = Command::new("open").arg(&project.path).status();
        }
    }

    pub fn analysis_scroll_down(&mut self, amount: u16) {
        self.analysis_scroll = self.analysis_scroll.saturating_add(amount);
    }

    pub fn analysis_scroll_up(&mut self, amount: u16) {
        self.analysis_scroll = self.analysis_scroll.saturating_sub(amount);
    }

    // ============ Key Handling ============

    pub fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) {
        match self.view {
            View::List => {
                if self.search_mode {
                    match code {
                        KeyCode::Esc => {
                            self.search_mode = false;
                            self.search_query.clear();
                            match self.tab {
                                Tab::Ideas => self.update_filter(),
                                Tab::Projects => self.update_project_filter(),
                            }
                        }
                        KeyCode::Enter => {
                            self.search_mode = false;
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                            match self.tab {
                                Tab::Ideas => self.update_filter(),
                                Tab::Projects => self.update_project_filter(),
                            }
                        }
                        KeyCode::Char(c) => {
                            self.search_query.push(c);
                            match self.tab {
                                Tab::Ideas => self.update_filter(),
                                Tab::Projects => self.update_project_filter(),
                            }
                        }
                        _ => {}
                    }
                } else {
                    match code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Tab => {
                            self.tab = self.tab.next();
                            self.search_query.clear();
                        }
                        KeyCode::Char('j') | KeyCode::Down => match self.tab {
                            Tab::Ideas => self.next(),
                            Tab::Projects => self.project_next(),
                        },
                        KeyCode::Char('k') | KeyCode::Up => match self.tab {
                            Tab::Ideas => self.previous(),
                            Tab::Projects => self.project_previous(),
                        },
                        KeyCode::Enter => match self.tab {
                            Tab::Ideas => self.enter_detail(),
                            Tab::Projects => self.open_analysis_preview(),
                        },
                        KeyCode::Char('s') => match self.tab {
                            Tab::Ideas => self.cycle_sort(),
                            Tab::Projects => self.cycle_project_sort(),
                        },
                        KeyCode::Char('f') => match self.tab {
                            Tab::Ideas => self.cycle_filter(),
                            Tab::Projects => {} // Could add category filter
                        },
                        KeyCode::Char('e') => self.open_in_editor(),
                        KeyCode::Char('o') => match self.tab {
                            Tab::Ideas => self.open_folder(),
                            Tab::Projects => self.open_project_folder(),
                        },
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
            View::AnalysisPreview => match code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Esc => self.view = View::List,
                KeyCode::Char('j') | KeyCode::Down => self.analysis_scroll_down(1),
                KeyCode::Char('k') | KeyCode::Up => self.analysis_scroll_up(1),
                KeyCode::Char('d') => self.analysis_scroll_down(10),
                KeyCode::Char('u') => self.analysis_scroll_up(10),
                KeyCode::Char('o') => self.open_project_folder(),
                KeyCode::PageDown => self.analysis_scroll_down(20),
                KeyCode::PageUp => self.analysis_scroll_up(20),
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

    pub fn project_stats(&self) -> (usize, usize) {
        let total = self.projects.len();
        let analyzed = self.projects.iter().filter(|p| has_analysis_file(&p.name)).count();
        (total, analyzed)
    }
}
