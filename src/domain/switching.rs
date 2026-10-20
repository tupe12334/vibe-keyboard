/// Tracks whether the current main page was chosen by the app in front (auto)
/// or by the user (manual), so that app switches never fight manual paging.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AutoPage {
    /// Page to return to when the app in front stops matching.
    manual: Option<usize>,
    /// True while the page shown was picked by the app in front.
    auto: bool,
}

impl AutoPage {
    /// The user paged by hand. The page is manual again from now on.
    pub const fn on_manual_page(&mut self) {
        self.auto = false;
        self.manual = None;
    }

    /// The profile now has `page_count` pages. Keep the remembered manual page in range.
    pub fn clamp(&mut self, page_count: usize) {
        if let Some(page) = self.manual {
            self.manual = Some(page.min(page_count.saturating_sub(1)));
        }
    }

    /// The app in front changed. `current` is the shown main page, `target` is the page
    /// that lists the new app (if any). Returns the page to show, if it must change.
    /// `on_main_page` is false while a sub-screen is open: nothing switches then.
    pub fn on_frontmost(
        &mut self,
        on_main_page: bool,
        current: usize,
        target: Option<usize>,
    ) -> Option<usize> {
        if !on_main_page {
            return None;
        }
        match target {
            Some(page) => {
                if !self.auto {
                    self.manual = Some(current);
                    self.auto = true;
                }
                (page != current).then_some(page)
            }
            None if self.auto => {
                self.auto = false;
                self.manual.take().filter(|page| *page != current)
            }
            None => None,
        }
    }
}

#[cfg(test)]
#[path = "switching_tests.rs"]
mod tests;
