use crate::domain::actions::{CentyIssue, CentyProject};

#[derive(Clone)]
pub enum Screen {
    MainPage {
        page: usize,
    },
    CentyProjectList {
        projects: Vec<CentyProject>,
        page: usize,
    },
    CentyProjectActions {
        project: CentyProject,
    },
    CentyIssueList {
        issues: Vec<CentyIssue>,
        page: usize,
        project_name: String,
    },
}

pub struct NavigationStack {
    stack: Vec<Screen>,
    total_main_pages: usize,
}

impl NavigationStack {
    pub fn new(initial_page: usize, total_main_pages: usize) -> Self {
        assert!(total_main_pages > 0, "must have at least one page");
        Self {
            stack: vec![Screen::MainPage {
                page: initial_page.min(total_main_pages - 1),
            }],
            total_main_pages,
        }
    }

    pub fn push(&mut self, screen: Screen) {
        self.stack.push(screen);
    }

    pub fn current(&self) -> &Screen {
        self.stack
            .last()
            .expect("NavigationStack is always non-empty")
    }

    /// Navigate back within the current screen's pages, or pop to the parent.
    /// `MainPage` wraps around (circular). Pushed screens go to the previous page
    /// if available, otherwise pop.
    pub fn back(&mut self) {
        let should_prev = match self
            .stack
            .last()
            .expect("NavigationStack is always non-empty")
        {
            Screen::MainPage { .. } => true,
            Screen::CentyProjectList { page, .. } => *page > 0,
            Screen::CentyIssueList { page, .. } => *page > 0,
            Screen::CentyProjectActions { .. } => false,
        };
        if should_prev {
            match self
                .stack
                .last_mut()
                .expect("NavigationStack is always non-empty")
            {
                Screen::MainPage { page } => {
                    *page = if *page == 0 {
                        self.total_main_pages - 1
                    } else {
                        *page - 1
                    };
                }
                Screen::CentyProjectList { page, .. } => *page -= 1,
                Screen::CentyIssueList { page, .. } => *page -= 1,
                Screen::CentyProjectActions { .. } => unreachable!(),
            }
        } else if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Pop to the parent screen. No-op at the bottom of the stack.
    pub fn out(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Navigate forward within the current screen's pages. No-op if no more pages.
    pub fn forward(&mut self) {
        match self
            .stack
            .last_mut()
            .expect("NavigationStack is always non-empty")
        {
            Screen::MainPage { page } => {
                *page = (*page + 1) % self.total_main_pages;
            }
            Screen::CentyProjectList { projects, page } => {
                let max = projects.len().saturating_sub(1) / 10;
                if *page < max {
                    *page += 1;
                }
            }
            Screen::CentyIssueList { issues, page, .. } => {
                let max = issues.len().saturating_sub(1) / 10;
                if *page < max {
                    *page += 1;
                }
            }
            Screen::CentyProjectActions { .. } => {}
        }
    }
}
