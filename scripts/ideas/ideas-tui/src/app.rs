use crate::data::{
    check_project_dirty, chrono_now, detect_untracked_projects_with_paths, find_markdown_files,
    get_project_head_commit, get_recent_activity_with_paths, dxitem_matches_query, has_analysis_file_with_paths,
    idea_matches_query, load_analysis_meta_with_paths, load_analysis_summary_with_paths,
    load_projects_with_paths, normalize_query, plan_matches_query, project_matches_query,
    save_analysis_meta_with_paths, sort_dotfiles, sort_ideas, sort_plans, sort_projects,
    DotfilesSortBy, DxItem, Idea, IdeasPaths, Plan, PlanSortBy, Project, ProjectAnalysisMeta,
    ProjectSortBy, RecentProject, SearchResult, SearchSource, SortBy, UntrackedProject,
};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::widgets::ListState;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

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

#[derive(Debug)]
pub struct FilteredList<T> {
    pub items: Vec<T>,
    pub filtered_indices: Vec<usize>,
    pub list_state: ListState,
}

impl<T> FilteredList<T> {
    pub fn new(items: Vec<T>) -> Self {
        let filtered_indices: Vec<usize> = (0..items.len()).collect();
        let mut list_state = ListState::default();
        if !filtered_indices.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            items,
            filtered_indices,
            list_state,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn filtered_len(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn filtered_items(&self) -> Vec<&T> {
        self.filtered_indices
            .iter()
            .map(|&i| &self.items[i])
            .collect()
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i))
            .map(|&idx| &self.items[idx])
    }

    pub fn apply_filter<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.filtered_indices = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| predicate(item))
            .map(|(i, _)| i)
            .collect();

        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
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
}

pub struct IdeasState {
    pub list: FilteredList<Idea>,
    pub sort_by: SortBy,
    pub filter: StatusFilter,
}

impl IdeasState {
    pub fn new(items: Vec<Idea>) -> Self {
        Self {
            list: FilteredList::new(items),
            sort_by: SortBy::Name,
            filter: StatusFilter::All,
        }
    }

    pub fn update_filter(&mut self, query: &str) {
        let q_lower = normalize_query(query);
        let filter = self.filter;
        self.list.apply_filter(|idea| {
            filter.matches(&idea.status) && idea_matches_query(idea, &q_lower)
        });
    }

    pub fn cycle_sort(&mut self, query: &str) {
        self.sort_by = self.sort_by.next();
        sort_ideas(&mut self.list.items, self.sort_by);
        self.update_filter(query);
    }

    pub fn cycle_filter(&mut self, query: &str) {
        self.filter = self.filter.next();
        self.update_filter(query);
    }
}

pub struct ProjectsState {
    pub list: FilteredList<Project>,
    pub sort_by: ProjectSortBy,
    pub detail_tab: ProjectDetailTab,
    pub info_scroll: u16,
    pub analysis_content: String,
    pub analysis_scroll: u16,
    pub analysis_total_lines: u16,
    pub is_busy: bool,
    pub busy_message: Option<String>,
    busy_message_deadline: Option<Instant>,
    update_rx: Option<Receiver<ProjectsUpdate>>,
}

impl ProjectsState {
    pub fn new(items: Vec<Project>) -> Self {
        Self {
            list: FilteredList::new(items),
            sort_by: ProjectSortBy::Analyzed,
            detail_tab: ProjectDetailTab::Info,
            info_scroll: 0,
            analysis_content: String::new(),
            analysis_scroll: 0,
            analysis_total_lines: 0,
            is_busy: false,
            busy_message: None,
            busy_message_deadline: None,
            update_rx: None,
        }
    }

    pub fn update_filter(&mut self, query: &str) {
        let q_lower = normalize_query(query);
        self.list
            .apply_filter(|project| project_matches_query(project, &q_lower));
    }

    pub fn cycle_sort(&mut self, query: &str) {
        self.sort_by = self.sort_by.next();
        sort_projects(&mut self.list.items, self.sort_by);
        self.update_filter(query);
    }
}

enum ProjectsUpdate {
    AnalyzeFinished {
        project_name: String,
        message: String,
        success: bool,
    },
    InventoryRefreshed {
        projects: Vec<Project>,
        message: String,
        success: bool,
    },
}

pub struct PlansState {
    pub list: FilteredList<Plan>,
    pub sort_by: PlanSortBy,
    pub content: String,
    pub scroll: u16,
    pub total_lines: u16,
}

impl PlansState {
    pub fn new(items: Vec<Plan>) -> Self {
        Self {
            list: FilteredList::new(items),
            sort_by: PlanSortBy::Modified,
            content: String::new(),
            scroll: 0,
            total_lines: 0,
        }
    }

    pub fn update_filter(&mut self, query: &str) {
        let q_lower = normalize_query(query);
        self.list
            .apply_filter(|plan| plan_matches_query(plan, &q_lower));
    }

    pub fn cycle_sort(&mut self, query: &str) {
        self.sort_by = self.sort_by.next();
        sort_plans(&mut self.list.items, self.sort_by);
        self.update_filter(query);
    }
}

pub struct DotfilesState {
    pub list: FilteredList<DxItem>,
    pub sort_by: DotfilesSortBy,
}

impl DotfilesState {
    pub fn new(items: Vec<DxItem>) -> Self {
        Self {
            list: FilteredList::new(items),
            sort_by: DotfilesSortBy::Category,
        }
    }

    pub fn update_filter(&mut self, query: &str) {
        let q_lower = normalize_query(query);
        self.list
            .apply_filter(|item| dxitem_matches_query(item, &q_lower));
    }

    pub fn cycle_sort(&mut self, query: &str) {
        self.sort_by = self.sort_by.next();
        sort_dotfiles(&mut self.list.items, self.sort_by);
        self.update_filter(query);
    }
}

pub struct StatusState {
    pub untracked_projects: Vec<UntrackedProject>,
    pub stale_projects: Vec<(String, u32)>, // (name, commits_since)
    pub recent_activity: Vec<RecentProject>,
    pub section: usize, // 0=untracked, 1=stale, 2=recent
    pub is_loading: bool,
    update_rx: Option<Receiver<StatusUpdate>>,
}

impl StatusState {
    pub fn new() -> Self {
        Self {
            untracked_projects: Vec::new(),
            stale_projects: Vec::new(),
            recent_activity: Vec::new(),
            section: 0,
            is_loading: false,
            update_rx: None,
        }
    }
}

struct StatusUpdate {
    untracked_projects: Vec<UntrackedProject>,
    stale_projects: Vec<(String, u32)>,
    recent_activity: Vec<RecentProject>,
}

pub struct MarkdownState {
    pub files: Vec<PathBuf>,
    pub list_state: ListState,
    pub content: String,
    pub scroll: u16,
    pub total_lines: u16,
}

impl Default for MarkdownState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            list_state: ListState::default(),
            content: String::new(),
            scroll: 0,
            total_lines: 0,
        }
    }
}

pub struct GlobalSearchState {
    pub results: Vec<SearchResult>,
    pub list_state: ListState,
}

impl Default for GlobalSearchState {
    fn default() -> Self {
        Self {
            results: Vec::new(),
            list_state: ListState::default(),
        }
    }
}

pub struct App {
    // General state
    pub tab: Tab,
    pub view: View,
    pub paths: IdeasPaths,
    pub repo_root: PathBuf,
    pub should_quit: bool,
    pub search_mode: bool,
    pub search_query: String,

    pub ideas: IdeasState,
    pub projects: ProjectsState,
    pub plans: PlansState,
    pub dotfiles: DotfilesState,
    pub status: StatusState,

    pub markdown: MarkdownState,
    pub global_search: GlobalSearchState,
}

impl App {
    pub fn new(
        paths: IdeasPaths,
        ideas: Vec<Idea>,
        projects: Vec<Project>,
        plans: Vec<Plan>,
        dotfiles: Vec<DxItem>,
        repo_root: PathBuf,
    ) -> Self {
        Self {
            tab: Tab::Ideas,
            view: View::List,
            paths,
            repo_root,
            should_quit: false,
            search_mode: false,
            search_query: String::new(),

            ideas: IdeasState::new(ideas),
            projects: ProjectsState::new(projects),
            plans: PlansState::new(plans),
            dotfiles: DotfilesState::new(dotfiles),
            status: StatusState::new(),

            markdown: MarkdownState::default(),
            global_search: GlobalSearchState::default(),
        }
    }

    pub fn tick(&mut self) {
        self.poll_status_refresh();
        self.poll_projects_task();
        self.tick_project_messages();
    }

    fn tick_project_messages(&mut self) {
        if let Some(deadline) = self.projects.busy_message_deadline {
            if Instant::now() >= deadline {
                self.projects.busy_message = None;
                self.projects.busy_message_deadline = None;
            }
        }
    }

    // ============ Ideas Methods ============

    pub fn filtered_ideas(&self) -> Vec<&Idea> {
        self.ideas.list.filtered_items()
    }

    pub fn selected_idea(&self) -> Option<&Idea> {
        self.ideas.list.selected_item()
    }

    pub fn update_filter(&mut self) {
        self.ideas.update_filter(&self.search_query);
    }

    pub fn cycle_sort(&mut self) {
        self.ideas.cycle_sort(&self.search_query);
    }

    pub fn cycle_filter(&mut self) {
        self.ideas.cycle_filter(&self.search_query);
    }

    pub fn next(&mut self) {
        self.ideas.list.next();
    }

    pub fn previous(&mut self) {
        self.ideas.list.previous();
    }

    pub fn enter_detail(&mut self) {
        if let Some(idea) = self.selected_idea() {
            let idea_path = self.repo_root.join(&idea.folder);
            self.markdown.files = find_markdown_files(&idea_path);
            self.markdown.list_state = ListState::default();
            if !self.markdown.files.is_empty() {
                self.markdown.list_state.select(Some(0));
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
        self.projects.list.filtered_items()
    }

    pub fn selected_project(&self) -> Option<&Project> {
        self.projects.list.selected_item()
    }

    pub fn update_project_filter(&mut self) {
        self.projects.update_filter(&self.search_query);
    }

    pub fn cycle_project_sort(&mut self) {
        self.projects.cycle_sort(&self.search_query);
    }

    pub fn project_next(&mut self) {
        self.projects.list.next();
    }

    pub fn project_previous(&mut self) {
        self.projects.list.previous();
    }

    pub fn open_project_detail(&mut self) {
        if self.selected_project().is_some() {
            self.projects.detail_tab = ProjectDetailTab::Info;
            self.projects.info_scroll = 0;
            // Load analysis if available
            if let Some(project) = self.selected_project() {
                if let Some(content) = load_analysis_summary_with_paths(&self.paths, &project.name) {
                    self.projects.analysis_total_lines = content.lines().count() as u16;
                    self.projects.analysis_content = content;
                } else {
                    self.projects.analysis_content = String::new();
                    self.projects.analysis_total_lines = 0;
                }
            }
            self.projects.analysis_scroll = 0;
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
        self.plans.list.filtered_items()
    }

    pub fn selected_plan(&self) -> Option<&Plan> {
        self.plans.list.selected_item()
    }

    pub fn update_plan_filter(&mut self) {
        self.plans.update_filter(&self.search_query);
    }

    pub fn cycle_plan_sort(&mut self) {
        self.plans.cycle_sort(&self.search_query);
    }

    pub fn plan_next(&mut self) {
        self.plans.list.next();
    }

    pub fn plan_previous(&mut self) {
        self.plans.list.previous();
    }

    pub fn open_plan_viewer(&mut self) {
        if let Some(plan) = self.selected_plan() {
            if let Ok(content) = std::fs::read_to_string(&plan.path) {
                self.plans.total_lines = content.lines().count() as u16;
                self.plans.content = content;
                self.plans.scroll = 0;
                self.view = View::PlanViewer;
            }
        }
    }

    // ============ Dotfiles Methods ============

    pub fn filtered_dotfiles(&self) -> Vec<&DxItem> {
        self.dotfiles.list.filtered_items()
    }

    pub fn selected_dotfile(&self) -> Option<&DxItem> {
        self.dotfiles.list.selected_item()
    }

    pub fn update_dotfiles_filter(&mut self) {
        self.dotfiles.update_filter(&self.search_query);
    }

    pub fn cycle_dotfiles_sort(&mut self) {
        self.dotfiles.cycle_sort(&self.search_query);
    }

    pub fn dotfiles_next(&mut self) {
        self.dotfiles.list.next();
    }

    pub fn dotfiles_previous(&mut self) {
        self.dotfiles.list.previous();
    }

    pub fn open_dotfile(&self) {
        if let Some(item) = self.selected_dotfile() {
            let _ = Command::new("open").arg(&item.path).status();
        }
    }

    // ============ Status Dashboard Methods ============

    pub fn refresh_status(&mut self) {
        if self.status.update_rx.is_some() {
            return;
        }

        self.status.is_loading = true;
        let paths = self.paths.clone();
        let projects = self.projects.list.items.clone();
        let (tx, rx) = mpsc::channel::<StatusUpdate>();
        self.status.update_rx = Some(rx);

        thread::spawn(move || {
            let untracked_projects =
                detect_untracked_projects_with_paths(&paths).unwrap_or_default();

            let mut stale_projects = Vec::new();
            if let Ok(meta) = load_analysis_meta_with_paths(&paths) {
                stale_projects = projects
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
                stale_projects.sort_by(|a, b| b.1.cmp(&a.1));
            }

            let recent_activity = get_recent_activity_with_paths(&paths, 7).unwrap_or_default();

            let _ = tx.send(StatusUpdate {
                untracked_projects,
                stale_projects,
                recent_activity,
            });
        });
    }

    pub fn status_next_section(&mut self) {
        self.status.section = (self.status.section + 1) % 3;
    }

    pub fn status_prev_section(&mut self) {
        self.status.section = if self.status.section == 0 {
            2
        } else {
            self.status.section - 1
        };
    }

    fn poll_status_refresh(&mut self) {
        enum PollResult {
            None,
            Update(StatusUpdate),
            Disconnected,
        }

        let result = match self.status.update_rx.as_ref() {
            Some(rx) => match rx.try_recv() {
                Ok(update) => PollResult::Update(update),
                Err(TryRecvError::Empty) => PollResult::None,
                Err(TryRecvError::Disconnected) => PollResult::Disconnected,
            },
            None => PollResult::None,
        };

        match result {
            PollResult::None => {}
            PollResult::Update(update) => {
                self.status.untracked_projects = update.untracked_projects;
                self.status.stale_projects = update.stale_projects;
                self.status.recent_activity = update.recent_activity;
                self.status.is_loading = false;
                self.status.update_rx = None;
            }
            PollResult::Disconnected => {
                self.status.is_loading = false;
                self.status.update_rx = None;
            }
        }
    }

    fn poll_projects_task(&mut self) {
        enum PollResult {
            None,
            Update(ProjectsUpdate),
            Disconnected,
        }

        let result = match self.projects.update_rx.as_ref() {
            Some(rx) => match rx.try_recv() {
                Ok(update) => PollResult::Update(update),
                Err(TryRecvError::Empty) => PollResult::None,
                Err(TryRecvError::Disconnected) => PollResult::Disconnected,
            },
            None => PollResult::None,
        };

        match result {
            PollResult::None => {}
            PollResult::Disconnected => {
                self.projects.is_busy = false;
                self.projects.update_rx = None;
            }
            PollResult::Update(update) => match update {
                ProjectsUpdate::AnalyzeFinished {
                    project_name,
                    message,
                    success: _,
                } => {
                    self.projects.is_busy = false;
                    self.projects.busy_message = Some(message);
                    self.projects.busy_message_deadline =
                        Some(Instant::now() + Duration::from_secs(3));
                    self.projects.update_rx = None;

                    if self.view == View::ProjectDetail {
                        if let Some(project) = self.selected_project() {
                            if project.name == project_name {
                                if let Some(content) =
                                    load_analysis_summary_with_paths(&self.paths, &project.name)
                                {
                                    self.projects.analysis_total_lines =
                                        content.lines().count() as u16;
                                    self.projects.analysis_content = content;
                                }
                            }
                        }
                    }

                    let selected_name = self.selected_project().map(|p| p.name.clone());
                    sort_projects(&mut self.projects.list.items, self.projects.sort_by);
                    self.update_project_filter();
                    if let Some(name) = selected_name {
                        self.select_project_by_name(&name);
                    }

                    self.refresh_status();
                }
                ProjectsUpdate::InventoryRefreshed {
                    projects,
                    message,
                    success: _,
                } => {
                    self.projects.is_busy = false;
                    self.projects.busy_message = Some(message);
                    self.projects.busy_message_deadline =
                        Some(Instant::now() + Duration::from_secs(3));
                    self.projects.update_rx = None;

                    let selected_name = self.selected_project().map(|p| p.name.clone());
                    self.projects.list.items = projects;
                    sort_projects(&mut self.projects.list.items, self.projects.sort_by);
                    self.update_project_filter();
                    if let Some(name) = selected_name {
                        self.select_project_by_name(&name);
                    }

                    self.refresh_status();
                }
            },
        }
    }

    fn select_project_by_name(&mut self, name: &str) {
        if let Some(item_index) = self.projects.list.items.iter().position(|p| p.name == name) {
            if let Some(filtered_pos) = self
                .projects
                .list
                .filtered_indices
                .iter()
                .position(|&i| i == item_index)
            {
                self.projects.list.list_state.select(Some(filtered_pos));
            }
        }
    }

    fn start_project_inventory_refresh(&mut self) {
        if self.projects.update_rx.is_some() {
            return;
        }

        self.projects.is_busy = true;
        self.projects.busy_message = Some("Refreshing inventory…".to_string());
        self.projects.busy_message_deadline = None;

        let paths = self.paths.clone();
        let (tx, rx) = mpsc::channel::<ProjectsUpdate>();
        self.projects.update_rx = Some(rx);

        thread::spawn(move || {
            let scan_script = paths
                .dotfiles_repo
                .join("scripts/ideas/mq/projects-scan.sh");

            let status = Command::new("bash")
                .arg(&scan_script)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            let (success, projects, message) = match status {
                Ok(status) if status.success() => match load_projects_with_paths(&paths) {
                    Ok(projects) => (true, projects, "Inventory refreshed".to_string()),
                    Err(err) => (false, Vec::new(), format!("Reload failed: {err}")),
                },
                Ok(_) => (false, Vec::new(), "Refresh failed".to_string()),
                Err(err) => (false, Vec::new(), format!("Refresh failed: {err}")),
            };

            let _ = tx.send(ProjectsUpdate::InventoryRefreshed {
                projects,
                message,
                success,
            });
        });
    }

    fn start_project_analysis(&mut self, deep: bool) {
        if self.projects.update_rx.is_some() {
            return;
        }

        let project = match self.selected_project() {
            Some(p) => p.clone(),
            None => return,
        };

        let mode_label = if deep { "Deep analyzing" } else { "Analyzing" };

        self.projects.is_busy = true;
        self.projects.busy_message = Some(format!("{mode_label} {}…", project.name));
        self.projects.busy_message_deadline = None;

        let paths = self.paths.clone();
        let (tx, rx) = mpsc::channel::<ProjectsUpdate>();
        self.projects.update_rx = Some(rx);

        thread::spawn(move || {
            let meta = match load_analysis_meta_with_paths(&paths) {
                Ok(meta) => meta,
                Err(err) => {
                    let _ = tx.send(ProjectsUpdate::AnalyzeFinished {
                        project_name: project.name.clone(),
                        message: format!("Load meta failed: {err}"),
                        success: false,
                    });
                    return;
                }
            };

            let dirty = check_project_dirty(&project, &meta);
            let needs_analysis =
                dirty.analyzed_at.is_none() || dirty.commits_since.unwrap_or(0) > 0;
            if !needs_analysis {
                let _ = tx.send(ProjectsUpdate::AnalyzeFinished {
                    project_name: project.name.clone(),
                    message: "Up to date".to_string(),
                    success: true,
                });
                return;
            }

            let script_name = if deep {
                "analyze-project-deep.sh"
            } else {
                "analyze-project.sh"
            };
            let analyze_script = paths.dotfiles_repo.join("scripts/ideas/mq").join(script_name);

            let status = Command::new("bash")
                .arg(&analyze_script)
                .arg(&project.path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            let status_ok = match status {
                Ok(status) => status.success(),
                Err(_) => false,
            };

            if !status_ok {
                let _ = tx.send(ProjectsUpdate::AnalyzeFinished {
                    project_name: project.name.clone(),
                    message: "Analysis failed".to_string(),
                    success: false,
                });
                return;
            }

            let current_commit = get_project_head_commit(&project.path).unwrap_or_default();
            let now = chrono_now();
            let mut meta = meta;

            meta.projects.insert(
                project.name.clone(),
                ProjectAnalysisMeta {
                    analyzed_at: now,
                    analyzed_commit: current_commit,
                },
            );

            if let Err(err) = save_analysis_meta_with_paths(&paths, &meta) {
                let _ = tx.send(ProjectsUpdate::AnalyzeFinished {
                    project_name: project.name.clone(),
                    message: format!("Save meta failed: {err}"),
                    success: false,
                });
                return;
            }

            let _ = tx.send(ProjectsUpdate::AnalyzeFinished {
                project_name: project.name.clone(),
                message: "Analysis complete".to_string(),
                success: true,
            });
        });
    }

    // ============ Markdown/Detail Methods ============

    pub fn md_next(&mut self) {
        if self.markdown.files.is_empty() {
            return;
        }
        let i = match self.markdown.list_state.selected() {
            Some(i) => (i + 1) % self.markdown.files.len(),
            None => 0,
        };
        self.markdown.list_state.select(Some(i));
    }

    pub fn md_previous(&mut self) {
        if self.markdown.files.is_empty() {
            return;
        }
        let i = match self.markdown.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.markdown.files.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.markdown.list_state.select(Some(i));
    }

    pub fn open_markdown_file(&mut self) {
        if let Some(idx) = self.markdown.list_state.selected() {
            if let Some(path) = self.markdown.files.get(idx) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    self.markdown.total_lines = content.lines().count() as u16;
                    self.markdown.content = content;
                    self.markdown.scroll = 0;
                    self.view = View::MarkdownReader;
                }
            }
        }
    }

    pub fn selected_md_file(&self) -> Option<&PathBuf> {
        self.markdown
            .list_state
            .selected()
            .and_then(|i| self.markdown.files.get(i))
    }

    // ============ Global Search Methods ============

    pub fn perform_global_search(&mut self) {
        let query = normalize_query(&self.search_query);
        self.global_search.results.clear();

        if query.is_empty() {
            return;
        }

        // Search ideas
        for (i, idea) in self.ideas.list.items.iter().enumerate() {
            if idea_matches_query(idea, &query) {
                self.global_search.results.push(SearchResult {
                    source: SearchSource::Ideas,
                    name: idea.folder.clone(),
                    description: idea.description.clone(),
                    index: i,
                });
            }
        }

        // Search projects
        for (i, project) in self.projects.list.items.iter().enumerate() {
            if project_matches_query(project, &query) {
                self.global_search.results.push(SearchResult {
                    source: SearchSource::Projects,
                    name: project.name.clone(),
                    description: if !project.summary.is_empty() {
                        project.summary.clone()
                    } else {
                        project.description.clone()
                    },
                    index: i,
                });
            }
        }

        // Search plans
        for (i, plan) in self.plans.list.items.iter().enumerate() {
            if plan_matches_query(plan, &query) {
                self.global_search.results.push(SearchResult {
                    source: SearchSource::Plans,
                    name: plan.name.clone(),
                    description: plan.title.clone(),
                    index: i,
                });
            }
        }

        // Search dotfiles
        for (i, item) in self.dotfiles.list.items.iter().enumerate() {
            if dxitem_matches_query(item, &query) {
                self.global_search.results.push(SearchResult {
                    source: SearchSource::Dotfiles,
                    name: item.name.clone(),
                    description: item.description.clone(),
                    index: i,
                });
            }
        }

        if !self.global_search.results.is_empty() {
            self.global_search.list_state.select(Some(0));
        }
    }

    pub fn search_results_next(&mut self) {
        if self.global_search.results.is_empty() {
            return;
        }
        let i = match self.global_search.list_state.selected() {
            Some(i) => (i + 1) % self.global_search.results.len(),
            None => 0,
        };
        self.global_search.list_state.select(Some(i));
    }

    pub fn search_results_previous(&mut self) {
        if self.global_search.results.is_empty() {
            return;
        }
        let i = match self.global_search.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.global_search.results.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.global_search.list_state.select(Some(i));
    }

    pub fn jump_to_search_result(&mut self) {
        if let Some(idx) = self.global_search.list_state.selected() {
            if let Some(result) = self.global_search.results.get(idx) {
                match result.source {
                    SearchSource::Ideas => {
                        self.tab = Tab::Ideas;
                        self.ideas.list.list_state.select(Some(result.index));
                    }
                    SearchSource::Projects => {
                        self.tab = Tab::Projects;
                        self.projects
                            .list
                            .list_state
                            .select(Some(result.index));
                    }
                    SearchSource::Plans => {
                        self.tab = Tab::Plans;
                        self.plans.list.list_state.select(Some(result.index));
                    }
                    SearchSource::Dotfiles => {
                        self.tab = Tab::Dotfiles;
                        self.dotfiles
                            .list
                            .list_state
                            .select(Some(result.index));
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
            self.global_search.results.clear();
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
            KeyCode::Char('a') => {
                if self.tab == Tab::Projects {
                    self.start_project_analysis(false);
                }
            }
            KeyCode::Char('A') => {
                if self.tab == Tab::Projects {
                    self.start_project_analysis(true);
                }
            }
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
                match self.tab {
                    Tab::Status => self.refresh_status(),
                    Tab::Projects => self.start_project_inventory_refresh(),
                    _ => {}
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
            KeyCode::Char('j') | KeyCode::Down => {
                self.markdown.scroll = self.markdown.scroll.saturating_add(1)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.markdown.scroll = self.markdown.scroll.saturating_sub(1)
            }
            KeyCode::Char('d') => self.markdown.scroll = self.markdown.scroll.saturating_add(10),
            KeyCode::Char('u') => self.markdown.scroll = self.markdown.scroll.saturating_sub(10),
            KeyCode::Char('g') => self.markdown.scroll = 0,
            KeyCode::Char('G') => {
                self.markdown.scroll = self.markdown.total_lines.saturating_sub(10)
            }
            KeyCode::PageDown => self.markdown.scroll = self.markdown.scroll.saturating_add(20),
            KeyCode::PageUp => self.markdown.scroll = self.markdown.scroll.saturating_sub(20),
            _ => {}
        }
    }

    fn handle_project_detail_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.view = View::List,
            KeyCode::Tab => self.projects.detail_tab = self.projects.detail_tab.next(),
            KeyCode::Char('a') => self.start_project_analysis(false),
            KeyCode::Char('A') => self.start_project_analysis(true),
            KeyCode::Char('j') | KeyCode::Down => {
                match self.projects.detail_tab {
                    ProjectDetailTab::Info => {
                        self.projects.info_scroll =
                            self.projects.info_scroll.saturating_add(1)
                    }
                    ProjectDetailTab::Analysis => {
                        self.projects.analysis_scroll =
                            self.projects.analysis_scroll.saturating_add(1)
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.projects.detail_tab {
                    ProjectDetailTab::Info => {
                        self.projects.info_scroll =
                            self.projects.info_scroll.saturating_sub(1)
                    }
                    ProjectDetailTab::Analysis => {
                        self.projects.analysis_scroll =
                            self.projects.analysis_scroll.saturating_sub(1)
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
                self.plans.scroll = self.plans.scroll.saturating_add(1)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.plans.scroll = self.plans.scroll.saturating_sub(1)
            }
            KeyCode::Char('d') => self.plans.scroll = self.plans.scroll.saturating_add(10),
            KeyCode::Char('u') => self.plans.scroll = self.plans.scroll.saturating_sub(10),
            KeyCode::Char('g') => self.plans.scroll = 0,
            KeyCode::Char('G') => {
                self.plans.scroll = self.plans.total_lines.saturating_sub(10)
            }
            KeyCode::PageDown => self.plans.scroll = self.plans.scroll.saturating_add(20),
            KeyCode::PageUp => self.plans.scroll = self.plans.scroll.saturating_sub(20),
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
            Tab::Status => self.status_enter(),
        }
    }

    fn status_enter(&mut self) {
        match self.status.section {
            0 => {
                if let Some(p) = self.status.untracked_projects.first() {
                    let _ = Command::new("open").arg(&p.path).status();
                }
            }
            1 => {
                if let Some(name) = self
                    .status
                    .stale_projects
                    .first()
                    .map(|(name, _)| name.clone())
                {
                    self.tab = Tab::Projects;
                    self.view = View::List;
                    self.search_query.clear();
                    self.update_project_filter();
                    self.select_project_by_name(&name);
                }
            }
            2 => {
                if let Some(name) = self
                    .status
                    .recent_activity
                    .first()
                    .map(|p| p.name.clone())
                {
                    self.tab = Tab::Projects;
                    self.view = View::List;
                    self.search_query.clear();
                    self.update_project_filter();
                    self.select_project_by_name(&name);
                }
            }
            _ => {}
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
        let total = self.ideas.list.items.len();
        let active = self
            .ideas
            .list
            .items
            .iter()
            .filter(|i| i.status == "active")
            .count();
        let dormant = self
            .ideas
            .list
            .items
            .iter()
            .filter(|i| i.status == "dormant")
            .count();
        let questions: usize = self
            .ideas
            .list
            .items
            .iter()
            .map(|i| i.open_questions.len())
            .sum();
        (total, active, dormant, questions)
    }

    pub fn project_stats(&self) -> (usize, usize) {
        let total = self.projects.list.items.len();
        let analyzed = self
            .projects
            .list
            .items
            .iter()
            .filter(|p| has_analysis_file_with_paths(&self.paths, &p.name))
            .count();
        (total, analyzed)
    }

    pub fn plan_stats(&self) -> usize {
        self.plans.list.items.len()
    }

    pub fn dotfiles_stats(&self) -> usize {
        self.dotfiles.list.items.len()
    }

    pub fn status_stats(&self) -> (usize, usize, usize) {
        (
            self.status.untracked_projects.len(),
            self.status.stale_projects.len(),
            self.status.recent_activity.len(),
        )
    }
}
