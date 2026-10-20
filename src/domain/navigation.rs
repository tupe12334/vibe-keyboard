#[cfg(test)]
use crate::domain::actions::{CentyIssue, CentyProject, ClaudeSession};
#[cfg(test)]
use crate::domain::moadim::{Flash, MoadimList, Routine};
#[cfg(test)]
use std::time::Instant;

mod back_target;
mod input_digits;
mod paging;
mod screen;
pub use back_target::{back_key_on, BackKey};
use paging::moadim_len;
pub use paging::{action_pages, prompt_index, PROMPTS_PER_PAGE};
pub use screen::Screen;
mod lists;
mod moadim;

/// Which screen is shown: stack depth, screen kind, and its page (0 when it has none).
pub type ScreenId = (usize, std::mem::Discriminant<Screen>, usize);

pub struct NavigationStack {
    stack: Vec<Screen>,
    total_main_pages: usize,
}

#[allow(clippy::expect_used, reason = "the stack is never empty")]
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

    /// Change the number of main pages after a profile reload, clamping pages that no longer exist.
    pub fn set_main_pages(&mut self, total: usize) {
        self.total_main_pages = total.max(1);
        for screen in &mut self.stack {
            if let Screen::MainPage { page } = screen {
                *page = (*page).min(self.total_main_pages - 1);
            }
        }
    }

    /// The shown main page, if the top screen is a main page.
    pub fn main_page(&self) -> Option<usize> {
        match self.stack.last() {
            Some(Screen::MainPage { page }) => Some(*page),
            _ => None,
        }
    }

    /// Identity of the shown screen. It changes on any navigation: push, pop, or a page turn.
    pub fn screen_id(&self) -> ScreenId {
        let top = self.current();
        let page = match top {
            Screen::MainPage { page }
            | Screen::CentyProjectList { page, .. }
            | Screen::CentyIssueList { page, .. }
            | Screen::ClaudeSessions { page, .. }
            | Screen::SessionActions { page, .. }
            | Screen::MoadimList { page, .. } => *page,
            _ => 0,
        };
        (self.stack.len(), std::mem::discriminant(top), page)
    }

    /// The main page the stack opened from. Built-in screens keep its side screen.
    pub fn home_page(&self) -> usize {
        self.stack
            .iter()
            .rev()
            .find_map(|screen| match screen {
                Screen::MainPage { page } => Some(*page),
                _ => None,
            })
            .unwrap_or(0)
    }

    /// Show main page `page` (0-based) if the top screen is a main page.
    pub fn go_to_page(&mut self, page: usize) {
        let last = self.total_main_pages - 1;
        if let Some(Screen::MainPage { page: current }) = self.stack.last_mut() {
            *current = page.min(last);
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

    pub const fn can_back() -> bool {
        true // back always navigates (prev page or pops to parent)
    }

    pub const fn can_out(&self) -> bool {
        self.stack.len() > 1
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

#[cfg(test)]
#[path = "navigation_tests/support.rs"]
mod support;

#[cfg(test)]
#[path = "navigation_tests/back_key.rs"]
mod back_key;
#[cfg(test)]
#[path = "navigation_tests/back_pages.rs"]
mod back_pages;
#[cfg(test)]
#[path = "navigation_tests/can_forward.rs"]
mod can_forward;
#[cfg(test)]
#[path = "navigation_tests/forward_actions.rs"]
mod forward_actions;
#[cfg(test)]
#[path = "navigation_tests/forward_pages.rs"]
mod forward_pages;
#[cfg(test)]
#[path = "navigation_tests/input_number.rs"]
mod input_number;
#[cfg(test)]
#[path = "navigation_tests/moadim_flash.rs"]
mod moadim_flash;
#[cfg(test)]
#[path = "navigation_tests/moadim_list.rs"]
mod moadim_list;
#[cfg(test)]
#[path = "navigation_tests/out_and_actions.rs"]
mod out_and_actions;
#[cfg(test)]
#[path = "navigation_tests/screen_id.rs"]
mod screen_id;
#[cfg(test)]
#[path = "navigation_tests/session_actions.rs"]
mod session_actions;
#[cfg(test)]
#[path = "navigation_tests/sessions.rs"]
mod sessions;
#[cfg(test)]
#[path = "navigation_tests/sort.rs"]
mod sort;
#[cfg(test)]
#[path = "navigation_tests/stack.rs"]
mod stack;
