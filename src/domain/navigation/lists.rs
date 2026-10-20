use crate::domain::actions::ClaudeSession;

use super::{action_pages, Screen};

#[allow(clippy::expect_used, reason = "the stack is never empty")]
impl super::NavigationStack {
    /// Replace the sessions on the open session list, keeping the page inside the new range.
    pub fn set_sessions(&mut self, new: Vec<ClaudeSession>) {
        if let Some(Screen::ClaudeSessions { sessions, page }) = self.stack.last_mut() {
            let max = new.len().saturating_sub(1) / 10;
            *page = (*page).min(max);
            *sessions = new;
        }
    }

    /// Turn the page of a session action page. Forward wraps to the first page.
    pub fn turn_action_page(&mut self, forward: bool, count: usize) {
        if let Some(Screen::SessionActions { page, .. }) = self.stack.last_mut() {
            *page = if forward {
                (*page + 1) % action_pages(count)
            } else {
                page.saturating_sub(1)
            };
        }
    }

    /// Keep open session action pages inside the range after the prompt list changes.
    pub fn set_action_prompts(&mut self, count: usize) {
        for screen in &mut self.stack {
            if let Screen::SessionActions { page, .. } = screen {
                *page = (*page).min(action_pages(count) - 1);
            }
        }
    }

    /// Go to the previous page of a session or routine list. Unlike `back`, it never pops the screen.
    pub fn previous_page(&mut self) {
        if let Some(Screen::ClaudeSessions { page, .. } | Screen::MoadimList { page, .. }) =
            self.stack.last_mut()
        {
            *page = page.saturating_sub(1);
        }
    }

    /// Toggle sort order of the current list screen.
    /// Projects are sorted by name; issues are sorted by number.
    /// Each call reverses the current order.
    pub fn toggle_sort(&mut self) {
        let top = self
            .stack
            .last_mut()
            .expect("NavigationStack is always non-empty");
        match top {
            Screen::CentyProjectList { projects, .. } => {
                let asc = projects.windows(2).all(|w| w[0].name <= w[1].name);
                if asc {
                    projects.sort_by(|a, b| b.name.cmp(&a.name));
                } else {
                    projects.sort_by(|a, b| a.name.cmp(&b.name));
                }
            }
            Screen::CentyIssueList { issues, .. } => {
                let desc = issues.windows(2).all(|w| w[0].number >= w[1].number);
                if desc {
                    issues.sort_by_key(|i| i.number);
                } else {
                    issues.sort_by_key(|i| std::cmp::Reverse(i.number));
                }
            }
            _ => {}
        }
    }
}
