use std::collections::VecDeque;

const LOG_CAPACITY: usize = 10;

pub struct AppState {
    pub current_page: usize,
    pub total_pages: usize,
    pub pressed_key: Option<u8>,
    pub log: VecDeque<String>,
}

impl AppState {
    pub fn new(total_pages: usize) -> Self {
        Self {
            current_page: 0,
            total_pages,
            pressed_key: None,
            log: VecDeque::with_capacity(LOG_CAPACITY),
        }
    }

    pub fn push_log(&mut self, msg: String) {
        if self.log.len() == LOG_CAPACITY {
            self.log.pop_front();
        }
        self.log.push_back(msg);
    }
}
