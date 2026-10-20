use crate::domain::actions::{CentyIssue, CentyProject, ClaudeSession};
use crate::domain::moadim::{Flash, MoadimList, Routine};
use crate::domain::profile::SideConfig;
use std::time::Instant;

#[derive(Clone)]
pub enum Screen {
    MainPage {
        page: usize,
    },
    CentyProjectList {
        projects: Vec<CentyProject>,
        page: usize,
        filter: Option<String>,
    },
    CentyProjectActions {
        project: CentyProject,
    },
    CentyIssueList {
        issues: Vec<CentyIssue>,
        page: usize,
        project_name: String,
        org: String,
        filter: Option<String>,
    },
    CentyIssueActions {
        issue: CentyIssue,
        project_name: String,
        org: String,
    },
    InputNumber {
        value: String,
    },
    ClaudeSessions {
        sessions: Vec<ClaudeSession>,
        page: usize,
    },
    NewSessionChoice,
    SessionActions {
        session: ClaudeSession,
        page: usize,
    },
    MoadimList {
        list: MoadimList,
        page: usize,
    },
    MoadimRoutine {
        routine: Routine,
        flash: Option<Flash>,
    },
}

/// Routines on one list page. Same page size as the other lists.
const fn moadim_len(list: &MoadimList) -> usize {
    match list {
        MoadimList::Online(routines) => routines.len(),
        MoadimList::Pending | MoadimList::Offline => 0,
    }
}

/// Prompt keys on a session action page: keys 1 to 10. Ten prompts fit on one page.
pub const PROMPTS_PER_PAGE: usize = 10;

/// Number of action pages for `count` prompts. Always at least one.
pub const fn action_pages(count: usize) -> usize {
    if count == 0 {
        1
    } else {
        count.div_ceil(PROMPTS_PER_PAGE)
    }
}

/// Index into the prompt list for `key` on `page`. None for a key with no prompt on that page.
pub const fn prompt_index(page: usize, key: u8, count: usize) -> Option<usize> {
    if key < 1 || key > 10 {
        return None;
    }
    let index = page * PROMPTS_PER_PAGE + (key as usize - 1);
    if index < count {
        Some(index)
    } else {
        None
    }
}

/// What key 11 does on a screen. `Back` steps within the main pages. `Out` leaves a sub-screen,
/// the same as key 12.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackKey {
    Back,
    Out,
}

/// The key 11 decision for `screen`. Main pages keep back. Every built-in sub-screen uses out.
pub const fn back_key_on(screen: &Screen) -> BackKey {
    match screen {
        Screen::MainPage { .. } => BackKey::Back,
        _ => BackKey::Out,
    }
}

impl Screen {
    /// Side content this screen declares. None keeps the side screen of the home page.
    pub fn side(&self) -> Option<SideConfig> {
        match self {
            Self::MainPage { .. }
            | Self::CentyProjectList { .. }
            | Self::CentyProjectActions { .. }
            | Self::CentyIssueList { .. }
            | Self::CentyIssueActions { .. }
            | Self::InputNumber { .. }
            | Self::ClaudeSessions { .. }
            | Self::SessionActions { .. }
            | Self::MoadimList { .. }
            | Self::MoadimRoutine { .. }
            | Self::NewSessionChoice => None,
        }
    }
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
        let should_pop = match self
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
        match self
            .stack
            .last()
            .expect("NavigationStack is always non-empty")
        {
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

    /// Update the open Moadim screen with a fresh list. Returns true when the screen changed.
    pub fn set_moadim(&mut self, new: MoadimList) -> bool {
        match self.stack.last_mut() {
            Some(Screen::MoadimList { list, page }) => {
                if *list == new {
                    return false;
                }
                let max = moadim_len(&new).saturating_sub(1) / 10;
                *page = (*page).min(max);
                *list = new;
                true
            }
            Some(Screen::MoadimRoutine { routine, .. }) => {
                let MoadimList::Online(routines) = &new else {
                    return false;
                };
                let Some(fresh) = routines.iter().find(|r| r.id == routine.id) else {
                    return false;
                };
                if fresh == routine {
                    return false;
                }
                *routine = fresh.clone();
                true
            }
            _ => false,
        }
    }

    /// True when the top screen shows Moadim routines.
    pub fn is_moadim(&self) -> bool {
        matches!(
            self.stack.last(),
            Some(Screen::MoadimList { .. } | Screen::MoadimRoutine { .. })
        )
    }

    /// Show `flash` on the open routine screen, replacing any flash already there.
    pub fn set_flash(&mut self, flash: Flash) {
        if let Some(Screen::MoadimRoutine { flash: slot, .. }) = self.stack.last_mut() {
            *slot = Some(flash);
        }
    }

    /// Drop the flash on the open routine screen once its time is up. True when it dropped one.
    pub fn clear_expired_flash(&mut self, now: Instant) -> bool {
        match self.stack.last_mut() {
            Some(Screen::MoadimRoutine { flash, .. })
                if flash.as_ref().is_some_and(|f| now >= f.until) =>
            {
                *flash = None;
                true
            }
            _ => false,
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
        match self
            .stack
            .last_mut()
            .expect("NavigationStack is always non-empty")
        {
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

    /// Append a digit character to the current `InputNumber` value.
    /// No-op if not on the `InputNumber` screen.
    pub fn input_number_append(&mut self, digit: char) {
        if let Some(Screen::InputNumber { value }) = self.stack.last_mut() {
            value.push(digit);
        }
    }

    /// Clear the `InputNumber` value.
    /// No-op if not on the `InputNumber` screen.
    pub fn input_number_clear(&mut self) {
        if let Some(Screen::InputNumber { value }) = self.stack.last_mut() {
            value.clear();
        }
    }

    /// Delete the last character of the `InputNumber` value (backspace).
    /// No-op if not on the `InputNumber` screen.
    pub fn input_number_backspace(&mut self) {
        if let Some(Screen::InputNumber { value }) = self.stack.last_mut() {
            value.pop();
        }
    }

    /// Return the current `InputNumber` value, or `None` if not on that screen.
    pub fn input_number_value(&self) -> Option<&str> {
        if let Some(Screen::InputNumber { value }) = self.stack.last() {
            Some(value)
        } else {
            None
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
