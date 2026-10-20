use super::{moadim_len, NavigationStack, Screen};

#[allow(clippy::expect_used, reason = "the stack is never empty")]
impl NavigationStack {
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
