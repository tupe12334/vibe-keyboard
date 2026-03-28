use std::collections::{HashMap, VecDeque};

use crate::application::page_actions;
use crate::domain::actions::{ButtonAction, CentyState};

const LOG_CAPACITY: usize = 10;

pub struct AppState {
    pub current_page: usize,
    pub total_pages: usize,
    pub pressed_key: Option<u8>,
    pub loading: bool,
    pub log: VecDeque<String>,
    pub actions: HashMap<u8, ButtonAction>,
    pub centy_state: Option<CentyState>,
}

impl AppState {
    pub fn new(total_pages: usize) -> Self {
        Self {
            current_page: 0,
            total_pages,
            pressed_key: None,
            loading: false,
            log: VecDeque::with_capacity(LOG_CAPACITY),
            actions: page_actions(0),
            centy_state: None,
        }
    }

    pub fn push_log(&mut self, msg: String) {
        if self.log.len() == LOG_CAPACITY {
            self.log.pop_front();
        }
        self.log.push_back(msg);
    }
}
