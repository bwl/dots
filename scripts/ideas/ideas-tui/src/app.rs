use crate::data::{
    check_project_dirty, detect_untracked_projects, find_markdown_files, get_recent_activity,
    has_analysis_file, load_analysis_meta, load_analysis_summary, sort_dotfiles, sort_ideas,
    sort_plans, sort_projects, DotfilesSortBy, DxItem, Idea, Plan, PlanSortBy, Project,
    ProjectSortBy, RecentProject, SearchResult, SearchSource, SortBy, UntrackedProject,
};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::widgets::ListState;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Ideas,
    Projects,
    Plans,
    Dotfiles,
    Status,
}

impl Tab {
    pub fn next(self) -> Self {
        match self {
            Tab::Ideas => Tab::Projects,
            Tab::Projects => Tab::Plans,
            Tab::Plans => Tab::Dotfiles,
            Tab::Dotfiles => Tab::Status,
            Tab::Status => Tab::Ideas,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Tab::Ideas => Tab::Status,
            Tab::Projects => Tab::Ideas,
            Tab::Plans => Tab::Projects,
            Tab::Dotfiles => Tab::Plans,
            Tab::Status => Tab::Dotfiles,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Tab::Ideas => "Ideas",
            Tab::Projects => "Projects",
            Tab::Plans => "Plans",
            Tab::Dotfiles => "Dotfiles",
            Tab::Status => "Status",
        }
    }

    pub fn all() -> Vec<Tab> {
        vec![Tab::Ideas, Tab::Projects, Tab::Plans, Tab::Dotfiles, Tab::Status]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    List,
    Detail,         // Idea detail (markdown files)
    MarkdownReader, // Reading a markdown file
    ProjectDetail,  // Project info/analysis tabs
    PlanViewer,     // Viewing a plan
    GlobalSearch,   // Cross-source search results
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectDetailTab {
    Info,
    Analysis,
}

impl ProjectDetailTab {
    pub fn next(self) -> Self {
        match self {
            ProjectDetailTab::Info => ProjectDetailTab::Analysis,
            ProjectDetailTab::Analysis => ProjectDetailTab::Info,
        }
    }
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
    // General state
    pub tab: Tab,
    pub view: View,
    pub repo_root: PathBuf,
    pub should_quit: bool,
    pub search_mode: bool,
    pub search_query: String,

    // Ideas state
    pub ideas: Vec<Idea>,
    pub filtered_indices: Vec<usize>,
    pub list_state: ListState,
    pub sort_by: SortBy,
    pub filter: StatusFilter,

    // Projects state
    pub projects: Vec<Project>,
    pub project_filtered_indices: Vec<usize>,
    pub project_list_state: ListState,
    pub project_sort_by: ProjectSortBy,
    pub project_detail_tab: ProjectDetailTab,
    pub project_info_scroll: u16,

    // Plans state
    pub plans: Vec<Plan>,
    pub plan_filtered_indices: Vec<usize>,
    pub plan_list_state: ListState,
    pub plan_sort_by: PlanSortBy,
    pub plan_content: String,
    pub plan_scroll: u16,
    pub plan_total_lines: u16,

    // Dotfiles state
    pub dotfiles: Vec<DxItem>,
    pub dotfiles_filtered_indices: Vec<usize>,
    pub dotfiles_list_state: ListState,
    pub dotfiles_sort_by: DotfilesSortBy,

    // Status dashboard state
    pub untracked_projects: Vec<UntrackedProject>,
    pub stale_projects: Vec<(String, u32)>, // (name, commits_since)
    pub recent_activity: Vec<RecentProject>,
    pub status_section: usize, // 0=untracked, 1=stale, 2=recent

    // Analysis/markdown viewer state
    pub analysis_content: String,
    pub analysis_scroll: u16,
    pub analysis_total_lines: u16,
    pub md_files: Vec<PathBuf>,
    pub md_list_state: ListState,
    pub md_content: String,
    pub md_scroll: u16,
    pub md_total_lines: u16,

    // Global search state
    pub search_results: Vec<SearchResult>,
    pub search_results_state: ListState,
}

impl App {
    pub fn new(
        ideas: Vec<Idea>,
        projects: Vec<Project>,
        plans: Vec<Plan>,
        dotfiles: Vec<DxItem>,
        repo_root: PathBuf,
    ) -> Self {
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

        let plan_filtered_indices: Vec<usize> = (0..plans.len()).collect();
        let mut plan_list_state = ListState::default();
        if !plan_filtered_indices.is_empty() {
            plan_list_state.select(Some(0));
        }

        let dotfiles_filtered_indices: Vec<usize> = (0..dotfiles.len()).collect();
        let mut dotfiles_list_state = ListState::default();
        if !dotfiles_filtered_indices.is_empty() {
            dotfiles_list_state.select(Some(0));
        }

        Self {
            tab: Tab::Ideas,
            view: View::List,
            repo_root,
            should_quit: false,
            search_mode: false,
            search_query: String::new(),

            ideas,
            filtered_indices,
            list_state,
            sort_by: SortBy::Name,
            filter: StatusFilter::All,

            projects,
            project_filtered_indices,
            project_list_state,
            project_sort_by: ProjectSortBy::Analyzed,
            project_detail_tab: ProjectDetailTab::Info,
            project_info_scroll: 0,

            plans,
            plan_filtered_indices,
            plan_list_state,
            plan_sort_by: PlanSortBy::Modified,
            plan_content: String::new(),
            plan_scroll: 0,
            plan_total_lines: 0,

            dotfiles,
            dotfiles_filtered_indices,
            dotfiles_list_state,
            dotfiles_sort_by: DotfilesSortBy::Category,

            untracked_projects: Vec::new(),
            stale_projects: Vec::new(),
            recent_activity: Vec::new(),
            status_section: 0,

            analysis_content: String::new(),
            analysis_scroll: 0,
            analysis_total_lines: 0,
            md_files: Vec::new(),
            md_list_state: ListState::default(),
            md_content: String::new(),
            md_scroll: 0,
            md_total_lines: 0,

            search_results: Vec::new(),
            search_results_state: ListState::default(),
        }
    }

    // ============ Ideas Methods ============

    pub fn filtered_ideas(&self) -> Vec<&Idea> {
        self.filtered_indices.iter().map(|&i| &self.ideas[i]).collect()
    }

    pub fn selected_idea(&self) -> Option<&Idea> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i))
            .map(|&idx| &self.ideas[idx])
    }

    fn matches_idea_search(&self, idea: &Idea) -> bool {
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
            .filter(|(_, idea)| self.filter.matches(&idea.status) && self.matches_idea_search(idea))
            .map(|(i, _)| i)
            .collect();

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
            Some(i) => (i + 1) % self.filtered_indices.len(),
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
                if i == 0 { self.filtered_indices.len() - 1 } else { i - 1 }
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
            let _ = Command::new("open").arg(&folder_path).status();
        }
    }

    // ============ Projects Methods ============

    pub fn filtered_projects(&self) -> Vec<&Project> {
        self.project_filtered_indices.iter().map(|&i| &self.projects[i]).collect()
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
                if !self.search_query.is_empty() {
                    let q = self.search_query.to_lowercase();
                    if !p.name.to_lowercase().contains(&q)
                        && !p.description.to_lowercase().contains(&q)
                        && !p.category.to_lowercase().contains(&q)
                        && !p.tech.to_lowercase().contains(&q)
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
            Some(i) => (i + 1) % self.project_filtered_indices.len(),
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
                if i == 0 { self.project_filtered_indices.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.project_list_state.select(Some(i));
    }

    pub fn open_project_detail(&mut self) {
        if self.selected_project().is_some() {
            self.project_detail_tab = ProjectDetailTab::Info;
            self.project_info_scroll = 0;
            // Load analysis if available
            if let Some(project) = self.selected_project() {
                if let Some(content) = load_analysis_summary(&project.name) {
                    self.analysis_total_lines = content.lines().count() as u16;
                    self.analysis_content = content;
                } else {
                    self.analysis_content = String::new();
                    self.analysis_total_lines = 0;
                }
            }
            self.analysis_scroll = 0;
            self.view = View::ProjectDetail;
        }
    }

    pub fn open_project_folder(&self) {
        if let Some(project) = self.selected_project() {
            let _ = Command::new("open").arg(&project.path).status();
        }
    }

    // ============ Plans Methods ============

    pub fn filtered_plans(&self) -> Vec<&Plan> {
        self.plan_filtered_indices.iter().map(|&i| &self.plans[i]).collect()
    }

    pub fn selected_plan(&self) -> Option<&Plan> {
        self.plan_list_state
            .selected()
            .and_then(|i| self.plan_filtered_indices.get(i))
            .map(|&idx| &self.plans[idx])
    }

    pub fn update_plan_filter(&mut self) {
        self.plan_filtered_indices = self
            .plans
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if !self.search_query.is_empty() {
                    let q = self.search_query.to_lowercase();
                    if !p.name.to_lowercase().contains(&q) && !p.title.to_lowercase().contains(&q) {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();

        if self.plan_filtered_indices.is_empty() {
            self.plan_list_state.select(None);
        } else {
            self.plan_list_state.select(Some(0));
        }
    }

    pub fn cycle_plan_sort(&mut self) {
        self.plan_sort_by = self.plan_sort_by.next();
        sort_plans(&mut self.plans, self.plan_sort_by);
        self.update_plan_filter();
    }

    pub fn plan_next(&mut self) {
        if self.plan_filtered_indices.is_empty() {
            return;
        }
        let i = match self.plan_list_state.selected() {
            Some(i) => (i + 1) % self.plan_filtered_indices.len(),
            None => 0,
        };
        self.plan_list_state.select(Some(i));
    }

    pub fn plan_previous(&mut self) {
        if self.plan_filtered_indices.is_empty() {
            return;
        }
        let i = match self.plan_list_state.selected() {
            Some(i) => {
                if i == 0 { self.plan_filtered_indices.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.plan_list_state.select(Some(i));
    }

    pub fn open_plan_viewer(&mut self) {
        if let Some(plan) = self.selected_plan() {
            if let Ok(content) = std::fs::read_to_string(&plan.path) {
                self.plan_total_lines = content.lines().count() as u16;
                self.plan_content = content;
                self.plan_scroll = 0;
                self.view = View::PlanViewer;
            }
        }
    }

    // ============ Dotfiles Methods ============

    pub fn filtered_dotfiles(&self) -> Vec<&DxItem> {
        self.dotfiles_filtered_indices.iter().map(|&i| &self.dotfiles[i]).collect()
    }

    pub fn selected_dotfile(&self) -> Option<&DxItem> {
        self.dotfiles_list_state
            .selected()
            .and_then(|i| self.dotfiles_filtered_indices.get(i))
            .map(|&idx| &self.dotfiles[idx])
    }

    pub fn update_dotfiles_filter(&mut self) {
        self.dotfiles_filtered_indices = self
            .dotfiles
            .iter()
            .enumerate()
            .filter(|(_, d)| {
                if !self.search_query.is_empty() {
                    let q = self.search_query.to_lowercase();
                    if !d.name.to_lowercase().contains(&q)
                        && !d.description.to_lowercase().contains(&q)
                        && !d.category.to_lowercase().contains(&q)
                    {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();

        if self.dotfiles_filtered_indices.is_empty() {
            self.dotfiles_list_state.select(None);
        } else {
            self.dotfiles_list_state.select(Some(0));
        }
    }

    pub fn cycle_dotfiles_sort(&mut self) {
        self.dotfiles_sort_by = self.dotfiles_sort_by.next();
        sort_dotfiles(&mut self.dotfiles, self.dotfiles_sort_by);
        self.update_dotfiles_filter();
    }

    pub fn dotfiles_next(&mut self) {
        if self.dotfiles_filtered_indices.is_empty() {
            return;
        }
        let i = match self.dotfiles_list_state.selected() {
            Some(i) => (i + 1) % self.dotfiles_filtered_indices.len(),
            None => 0,
        };
        self.dotfiles_list_state.select(Some(i));
    }

    pub fn dotfiles_previous(&mut self) {
        if self.dotfiles_filtered_indices.is_empty() {
            return;
        }
        let i = match self.dotfiles_list_state.selected() {
            Some(i) => {
                if i == 0 { self.dotfiles_filtered_indices.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.dotfiles_list_state.select(Some(i));
    }

    pub fn open_dotfile(&self) {
        if let Some(item) = self.selected_dotfile() {
            let _ = Command::new("open").arg(&item.path).status();
        }
    }

    // ============ Status Dashboard Methods ============

    pub fn refresh_status(&mut self) {
        // Load untracked projects
        self.untracked_projects = detect_untracked_projects().unwrap_or_default();

        // Load stale projects
        if let Ok(meta) = load_analysis_meta() {
            self.stale_projects = self
                .projects
                .iter()
                .filter_map(|p| {
                    let dirty = check_project_dirty(p, &meta);
                    if let Some(commits) = dirty.commits_since {
                        if commits > 0 {
                            return Some((p.name.clone(), commits));
                        }
                    }
                    None
                })
                .collect();
            self.stale_projects.sort_by(|a, b| b.1.cmp(&a.1));
        }

        // Load recent activity
        self.recent_activity = get_recent_activity(7).unwrap_or_default();
    }

    pub fn status_next_section(&mut self) {
        self.status_section = (self.status_section + 1) % 3;
    }

    pub fn status_prev_section(&mut self) {
        self.status_section = if self.status_section == 0 { 2 } else { self.status_section - 1 };
    }

    // ============ Markdown/Detail Methods ============

    pub fn md_next(&mut self) {
        if self.md_files.is_empty() {
            return;
        }
        let i = match self.md_list_state.selected() {
            Some(i) => (i + 1) % self.md_files.len(),
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
                if i == 0 { self.md_files.len() - 1 } else { i - 1 }
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
        self.md_list_state.selected().and_then(|i| self.md_files.get(i))
    }

    // ============ Global Search Methods ============

    pub fn perform_global_search(&mut self) {
        let query = self.search_query.to_lowercase();
        self.search_results.clear();

        if query.is_empty() {
            return;
        }

        // Search ideas
        for (i, idea) in self.ideas.iter().enumerate() {
            if idea.folder.to_lowercase().contains(&query)
                || idea.description.to_lowercase().contains(&query)
            {
                self.search_results.push(SearchResult {
                    source: SearchSource::Ideas,
                    name: idea.folder.clone(),
                    description: idea.description.clone(),
                    index: i,
                });
            }
        }

        // Search projects
        for (i, project) in self.projects.iter().enumerate() {
            if project.name.to_lowercase().contains(&query)
                || project.description.to_lowercase().contains(&query)
            {
                self.search_results.push(SearchResult {
                    source: SearchSource::Projects,
                    name: project.name.clone(),
                    description: project.description.clone(),
                    index: i,
                });
            }
        }

        // Search plans
        for (i, plan) in self.plans.iter().enumerate() {
            if plan.name.to_lowercase().contains(&query) || plan.title.to_lowercase().contains(&query)
            {
                self.search_results.push(SearchResult {
                    source: SearchSource::Plans,
                    name: plan.name.clone(),
                    description: plan.title.clone(),
                    index: i,
                });
            }
        }

        // Search dotfiles
        for (i, item) in self.dotfiles.iter().enumerate() {
            if item.name.to_lowercase().contains(&query)
                || item.description.to_lowercase().contains(&query)
            {
                self.search_results.push(SearchResult {
                    source: SearchSource::Dotfiles,
                    name: item.name.clone(),
                    description: item.description.clone(),
                    index: i,
                });
            }
        }

        if !self.search_results.is_empty() {
            self.search_results_state.select(Some(0));
        }
    }

    pub fn search_results_next(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        let i = match self.search_results_state.selected() {
            Some(i) => (i + 1) % self.search_results.len(),
            None => 0,
        };
        self.search_results_state.select(Some(i));
    }

    pub fn search_results_previous(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        let i = match self.search_results_state.selected() {
            Some(i) => {
                if i == 0 { self.search_results.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.search_results_state.select(Some(i));
    }

    pub fn jump_to_search_result(&mut self) {
        if let Some(idx) = self.search_results_state.selected() {
            if let Some(result) = self.search_results.get(idx) {
                match result.source {
                    SearchSource::Ideas => {
                        self.tab = Tab::Ideas;
                        self.list_state.select(Some(result.index));
                    }
                    SearchSource::Projects => {
                        self.tab = Tab::Projects;
                        self.project_list_state.select(Some(result.index));
                    }
                    SearchSource::Plans => {
                        self.tab = Tab::Plans;
                        self.plan_list_state.select(Some(result.index));
                    }
                    SearchSource::Dotfiles => {
                        self.tab = Tab::Dotfiles;
                        self.dotfiles_list_state.select(Some(result.index));
                    }
                }
                self.view = View::List;
                self.search_query.clear();
            }
        }
    }

    // ============ Key Handling ============

    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match self.view {
            View::List => self.handle_list_keys(code, modifiers),
            View::Detail => self.handle_detail_keys(code),
            View::MarkdownReader => self.handle_reader_keys(code),
            View::ProjectDetail => self.handle_project_detail_keys(code),
            View::PlanViewer => self.handle_plan_viewer_keys(code),
            View::GlobalSearch => self.handle_global_search_keys(code),
        }
    }

    fn handle_list_keys(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        if self.search_mode {
            match code {
                KeyCode::Esc => {
                    self.search_mode = false;
                    self.search_query.clear();
                    self.update_current_filter();
                }
                KeyCode::Enter => {
                    self.search_mode = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.update_current_filter();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.update_current_filter();
                }
                _ => {}
            }
            return;
        }

        // Ctrl+F for global search
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('f') {
            self.search_query.clear();
            self.search_results.clear();
            self.search_mode = true;
            self.view = View::GlobalSearch;
            return;
        }

        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => {
                self.tab = self.tab.next();
                self.search_query.clear();
                if self.tab == Tab::Status {
                    self.refresh_status();
                }
            }
            KeyCode::BackTab => {
                self.tab = self.tab.prev();
                self.search_query.clear();
                if self.tab == Tab::Status {
                    self.refresh_status();
                }
            }
            KeyCode::Char('j') | KeyCode::Down => self.current_next(),
            KeyCode::Char('k') | KeyCode::Up => self.current_previous(),
            KeyCode::Enter => self.current_enter(),
            KeyCode::Char('s') => self.current_cycle_sort(),
            KeyCode::Char('f') => {
                if self.tab == Tab::Ideas {
                    self.cycle_filter();
                }
            }
            KeyCode::Char('e') => {
                if self.tab == Tab::Ideas {
                    self.open_in_editor();
                }
            }
            KeyCode::Char('o') => self.current_open_folder(),
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
            }
            KeyCode::Char('r') => {
                if self.tab == Tab::Status {
                    self.refresh_status();
                }
            }
            _ => {}
        }
    }

    fn handle_detail_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.view = View::List,
            KeyCode::Char('j') | KeyCode::Down => self.md_next(),
            KeyCode::Char('k') | KeyCode::Up => self.md_previous(),
            KeyCode::Enter => self.open_markdown_file(),
            KeyCode::Char('e') => self.open_in_editor(),
            KeyCode::Char('o') => self.open_folder(),
            _ => {}
        }
    }

    fn handle_reader_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.view = View::Detail,
            KeyCode::Char('j') | KeyCode::Down => self.md_scroll = self.md_scroll.saturating_add(1),
            KeyCode::Char('k') | KeyCode::Up => self.md_scroll = self.md_scroll.saturating_sub(1),
            KeyCode::Char('d') => self.md_scroll = self.md_scroll.saturating_add(10),
            KeyCode::Char('u') => self.md_scroll = self.md_scroll.saturating_sub(10),
            KeyCode::Char('g') => self.md_scroll = 0,
            KeyCode::Char('G') => self.md_scroll = self.md_total_lines.saturating_sub(10),
            KeyCode::PageDown => self.md_scroll = self.md_scroll.saturating_add(20),
            KeyCode::PageUp => self.md_scroll = self.md_scroll.saturating_sub(20),
            _ => {}
        }
    }

    fn handle_project_detail_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.view = View::List,
            KeyCode::Tab => self.project_detail_tab = self.project_detail_tab.next(),
            KeyCode::Char('j') | KeyCode::Down => {
                match self.project_detail_tab {
                    ProjectDetailTab::Info => {
                        self.project_info_scroll = self.project_info_scroll.saturating_add(1)
                    }
                    ProjectDetailTab::Analysis => {
                        self.analysis_scroll = self.analysis_scroll.saturating_add(1)
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.project_detail_tab {
                    ProjectDetailTab::Info => {
                        self.project_info_scroll = self.project_info_scroll.saturating_sub(1)
                    }
                    ProjectDetailTab::Analysis => {
                        self.analysis_scroll = self.analysis_scroll.saturating_sub(1)
                    }
                }
            }
            KeyCode::Char('o') => self.open_project_folder(),
            _ => {}
        }
    }

    fn handle_plan_viewer_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.view = View::List,
            KeyCode::Char('j') | KeyCode::Down => {
                self.plan_scroll = self.plan_scroll.saturating_add(1)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.plan_scroll = self.plan_scroll.saturating_sub(1)
            }
            KeyCode::Char('d') => self.plan_scroll = self.plan_scroll.saturating_add(10),
            KeyCode::Char('u') => self.plan_scroll = self.plan_scroll.saturating_sub(10),
            KeyCode::Char('g') => self.plan_scroll = 0,
            KeyCode::Char('G') => self.plan_scroll = self.plan_total_lines.saturating_sub(10),
            KeyCode::PageDown => self.plan_scroll = self.plan_scroll.saturating_add(20),
            KeyCode::PageUp => self.plan_scroll = self.plan_scroll.saturating_sub(20),
            _ => {}
        }
    }

    fn handle_global_search_keys(&mut self, code: KeyCode) {
        if self.search_mode {
            match code {
                KeyCode::Esc => {
                    self.search_mode = false;
                    self.view = View::List;
                    self.search_query.clear();
                }
                KeyCode::Enter => {
                    self.search_mode = false;
                    self.perform_global_search();
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.perform_global_search();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.perform_global_search();
                }
                _ => {}
            }
        } else {
            match code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Esc => {
                    self.view = View::List;
                    self.search_query.clear();
                }
                KeyCode::Char('j') | KeyCode::Down => self.search_results_next(),
                KeyCode::Char('k') | KeyCode::Up => self.search_results_previous(),
                KeyCode::Enter => self.jump_to_search_result(),
                KeyCode::Char('/') => {
                    self.search_mode = true;
                }
                _ => {}
            }
        }
    }

    // Helper methods for current tab operations
    fn update_current_filter(&mut self) {
        match self.tab {
            Tab::Ideas => self.update_filter(),
            Tab::Projects => self.update_project_filter(),
            Tab::Plans => self.update_plan_filter(),
            Tab::Dotfiles => self.update_dotfiles_filter(),
            Tab::Status => {}
        }
    }

    fn current_next(&mut self) {
        match self.tab {
            Tab::Ideas => self.next(),
            Tab::Projects => self.project_next(),
            Tab::Plans => self.plan_next(),
            Tab::Dotfiles => self.dotfiles_next(),
            Tab::Status => self.status_next_section(),
        }
    }

    fn current_previous(&mut self) {
        match self.tab {
            Tab::Ideas => self.previous(),
            Tab::Projects => self.project_previous(),
            Tab::Plans => self.plan_previous(),
            Tab::Dotfiles => self.dotfiles_previous(),
            Tab::Status => self.status_prev_section(),
        }
    }

    fn current_enter(&mut self) {
        match self.tab {
            Tab::Ideas => self.enter_detail(),
            Tab::Projects => self.open_project_detail(),
            Tab::Plans => self.open_plan_viewer(),
            Tab::Dotfiles => self.open_dotfile(),
            Tab::Status => {} // Could navigate to relevant tab
        }
    }

    fn current_cycle_sort(&mut self) {
        match self.tab {
            Tab::Ideas => self.cycle_sort(),
            Tab::Projects => self.cycle_project_sort(),
            Tab::Plans => self.cycle_plan_sort(),
            Tab::Dotfiles => self.cycle_dotfiles_sort(),
            Tab::Status => {}
        }
    }

    fn current_open_folder(&self) {
        match self.tab {
            Tab::Ideas => self.open_folder(),
            Tab::Projects => self.open_project_folder(),
            Tab::Dotfiles => self.open_dotfile(),
            _ => {}
        }
    }

    // ============ Stats Methods ============

    pub fn idea_stats(&self) -> (usize, usize, usize, usize) {
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

    pub fn plan_stats(&self) -> usize {
        self.plans.len()
    }

    pub fn dotfiles_stats(&self) -> usize {
        self.dotfiles.len()
    }

    pub fn status_stats(&self) -> (usize, usize, usize) {
        (
            self.untracked_projects.len(),
            self.stale_projects.len(),
            self.recent_activity.len(),
        )
    }
}
