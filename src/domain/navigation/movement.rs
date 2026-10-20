use super::{moadim_len, NavigationStack, Screen};

#[allow(clippy::expect_used, reason = "the stack is never empty")]
impl NavigationStack {
    /// Navigate back within the current screen's pages, or pop to the parent.
    /// `MainPage` wraps around (circular). Pushed screens go to the previous page
    /// if available, otherwise pop.
    pub fn back(&mut self) {
        let top = self
            .stack
            .last_mut()
            .expect("NavigationStack is always non-empty");
        let should_pop = match top {
            Screen::MainPage { page } => {
                *page = if *page == 0 {
                    self.total_main_pages - 1
                } else {
                    *page - 1
                };
                false
            }
            Screen::CentyProjectList { page, .. }
            | Screen::CentyIssueList { page, .. }
            | Screen::ClaudeSessions { page, .. }
            | Screen::MoadimList { page, .. }
                if *page > 0 =>
            {
                *page -= 1;
                false
            }
            _ => true,
        };
        if should_pop && self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Pop to the parent screen. No-op at the bottom of the stack.
    pub fn out(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Drop every screen above the bottom main page. No-op when already there.
    pub fn out_to_main(&mut self) {
        self.stack.truncate(1);
    }

    pub fn can_forward(&self) -> bool {
        match self.current() {
            Screen::MainPage { .. } => true,
            Screen::CentyProjectList { projects, page, .. } => {
                *page < projects.len().saturating_sub(1) / 10
            }
            Screen::CentyIssueList { issues, page, .. } => {
                *page < issues.len().saturating_sub(1) / 10
            }
            Screen::ClaudeSessions { sessions, page } => {
                *page < sessions.len().saturating_sub(1) / 10
            }
            Screen::MoadimList { list, page } => *page < moadim_len(list).saturating_sub(1) / 10,
            Screen::CentyProjectActions { .. }
            | Screen::CentyIssueActions { .. }
            | Screen::InputNumber { .. }
            | Screen::NewSessionChoice
            | Screen::SessionActions { .. }
            | Screen::MoadimRoutine { .. } => false,
        }
    }

    /// Navigate forward within the current screen's pages. No-op if no more pages.
    pub fn forward(&mut self) {
        let top = self
            .stack
            .last_mut()
            .expect("NavigationStack is always non-empty");
        match top {
            Screen::MainPage { page } => {
                *page = (*page + 1) % self.total_main_pages;
            }
            Screen::CentyProjectList { projects, page, .. } => {
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
            Screen::ClaudeSessions { sessions, page } => {
                let max = sessions.len().saturating_sub(1) / 10;
                if *page < max {
                    *page += 1;
                }
            }
            Screen::MoadimList { list, page } => {
                let max = moadim_len(list).saturating_sub(1) / 10;
                if *page < max {
                    *page += 1;
                }
            }
            Screen::CentyProjectActions { .. }
            | Screen::CentyIssueActions { .. }
            | Screen::InputNumber { .. }
            | Screen::NewSessionChoice
            | Screen::SessionActions { .. }
            | Screen::MoadimRoutine { .. } => {}
        }
    }
}
