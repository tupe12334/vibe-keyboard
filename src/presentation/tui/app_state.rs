use std::collections::{HashMap, VecDeque};

use crate::domain::actions::{ButtonAction, ScreenView};
use throbber_widgets_tui::ThrobberState;

const LOG_CAPACITY: usize = 10;

pub struct AppState {
    pub total_pages: usize,
    pub pressed_key: Option<u8>,
    pub loading: bool,
    pub log: VecDeque<String>,
    pub actions: HashMap<u8, ButtonAction>,
    pub screen: ScreenView,
    pub nav_can_back: bool,
    pub nav_can_out: bool,
    pub nav_can_forward: bool,
    pub throbber_state: ThrobberState,
}

impl AppState {
    pub fn new(total_pages: usize) -> Self {
        Self {
            total_pages,
            pressed_key: None,
            loading: false,
            log: VecDeque::with_capacity(LOG_CAPACITY),
            actions: HashMap::new(),
            screen: ScreenView::MainPage { page: 0 },
            nav_can_back: true,
            nav_can_out: false,
            nav_can_forward: true,
            throbber_state: ThrobberState::default(),
        }
    }

    pub fn push_log(&mut self, msg: String) {
        if self.log.len() == LOG_CAPACITY {
            self.log.pop_front();
        }
        self.log.push_back(msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_defaults() {
        let s = AppState::new(3);
        assert_eq!(s.total_pages, 3);
        assert_eq!(s.pressed_key, None);
        assert!(!s.loading);
        assert!(s.log.is_empty());
        assert!(s.actions.is_empty());
        assert!(s.nav_can_back);
        assert!(!s.nav_can_out);
        assert!(s.nav_can_forward);
    }

    #[test]
    fn push_log_adds_entries() {
        let mut s = AppState::new(1);
        s.push_log("hello".into());
        assert_eq!(s.log.len(), 1);
        assert_eq!(s.log[0], "hello");
    }

    #[test]
    fn push_log_evicts_oldest_at_capacity() {
        let mut s = AppState::new(1);
        for i in 0..10 {
            s.push_log(format!("msg{i}"));
        }
        assert_eq!(s.log.len(), 10);
        // adding one more should evict the first
        s.push_log("overflow".into());
        assert_eq!(s.log.len(), 10);
        assert_eq!(s.log[0], "msg1");
        assert_eq!(s.log[9], "overflow");
    }
}
