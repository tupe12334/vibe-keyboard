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
mod tests {
    use super::*;

    #[test]
    fn switches_to_matching_page_and_back() {
        let mut state = AutoPage::default();
        assert_eq!(state.on_frontmost(true, 0, Some(2)), Some(2));
        assert_eq!(state.on_frontmost(true, 2, None), Some(0));
    }

    #[test]
    fn manual_paging_is_not_undone_when_no_app_matches() {
        let mut state = AutoPage::default();
        assert_eq!(state.on_frontmost(true, 0, None), None);
        state.on_manual_page();
        assert_eq!(state.on_frontmost(true, 1, None), None);
    }

    #[test]
    fn manual_page_after_auto_switch_becomes_the_base() {
        let mut state = AutoPage::default();
        assert_eq!(state.on_frontmost(true, 0, Some(2)), Some(2));
        state.on_manual_page();
        assert_eq!(state.on_frontmost(true, 2, None), None);
    }

    #[test]
    fn clamp_pulls_a_stale_manual_page_into_range() {
        let mut state = AutoPage::default();
        assert_eq!(state.on_frontmost(true, 4, Some(0)), Some(0));
        state.clamp(2);
        assert_eq!(state.on_frontmost(true, 0, None), Some(1));
    }

    #[test]
    fn sub_screen_blocks_switching() {
        let mut state = AutoPage::default();
        assert_eq!(state.on_frontmost(false, 0, Some(2)), None);
        assert_eq!(state.on_frontmost(true, 0, Some(2)), Some(2));
        assert_eq!(state.on_frontmost(false, 2, None), None);
        assert_eq!(state.on_frontmost(true, 2, None), Some(0));
    }

    #[test]
    fn switching_between_matching_apps_keeps_the_original_base() {
        let mut state = AutoPage::default();
        assert_eq!(state.on_frontmost(true, 0, Some(2)), Some(2));
        assert_eq!(state.on_frontmost(true, 2, Some(3)), Some(3));
        assert_eq!(state.on_frontmost(true, 3, None), Some(0));
    }
}
