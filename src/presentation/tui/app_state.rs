use std::collections::{HashMap, VecDeque};

use crate::domain::actions::{ButtonAction, ScreenView};

const LOG_CAPACITY: usize = 10;

pub struct AppState {
    pub total_pages: usize,
    pub pressed_key: Option<u8>,
    pub loading: bool,
    pub log: VecDeque<String>,
    pub actions: HashMap<u8, ButtonAction>,
    pub screen: ScreenView,
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
        }
    }

    pub fn push_log(&mut self, msg: String) {
        if self.log.len() == LOG_CAPACITY {
            self.log.pop_front();
        }
        self.log.push_back(msg);
    }
}
