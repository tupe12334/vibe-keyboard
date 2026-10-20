use super::{NavigationStack, Screen, ScreenId};

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

    pub const fn can_back() -> bool {
        true // back always navigates (prev page or pops to parent)
    }

    pub const fn can_out(&self) -> bool {
        self.stack.len() > 1
    }
}
