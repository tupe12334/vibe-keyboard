use std::collections::{HashMap, VecDeque};

use crate::domain::actions::{ButtonAction, ScreenView};
use crate::domain::hold::HoldTracker;
use crate::domain::profile::Profile;
use crate::domain::switching::AutoPage;
use crate::infrastructure::system_metrics::Metrics;
use throbber_widgets_tui::ThrobberState;

const LOG_CAPACITY: usize = 10;

pub struct AppState {
    pub total_pages: usize,
    /// The user's layout. Main-page keys and actions come from here.
    pub profile: Profile,
    /// Latest system sample, refreshed by the controller tick. Side widgets read it.
    pub metrics: Metrics,
    /// Side frame last sent to the device. Cleared on every full redraw.
    pub side_frame: Option<Vec<u8>>,
    /// Widget text last drawn on the side screen. Unchanged text skips the redraw.
    pub side_texts: Option<Vec<String>>,
    /// Whether the shown main page was picked by the app in front.
    pub auto_page: AutoPage,
    pub pressed_key: Option<u8>,
    /// Keys down now, to tell a tap from a hold.
    pub holds: HoldTracker,
    pub loading: bool,
    pub log: VecDeque<String>,
    pub actions: HashMap<u8, ButtonAction>,
    pub screen: ScreenView,
    pub nav_can_back: bool,
    pub nav_can_out: bool,
    pub nav_can_forward: bool,
    pub throbber_state: ThrobberState,
    pub text_input_mode: bool,
    pub text_input_value: String,
    pub text_input_confirmed: bool,
}

impl AppState {
    pub fn new(total_pages: usize) -> Self {
        Self {
            total_pages,
            profile: Profile::default(),
            metrics: Metrics::default(),
            side_frame: None,
            side_texts: None,
            auto_page: AutoPage::default(),
            pressed_key: None,
            holds: HoldTracker::default(),
            loading: false,
            log: VecDeque::with_capacity(LOG_CAPACITY),
            actions: HashMap::new(),
            screen: ScreenView::MainPage { page: 0 },
            nav_can_back: true,
            nav_can_out: false,
            nav_can_forward: true,
            throbber_state: ThrobberState::default(),
            text_input_mode: false,
            text_input_value: String::new(),
            text_input_confirmed: false,
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
#[path = "app_state_tests.rs"]
mod tests;
