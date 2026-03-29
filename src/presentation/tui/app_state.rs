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
