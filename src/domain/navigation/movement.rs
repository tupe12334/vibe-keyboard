use super::{NavigationStack, Screen};

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
}
