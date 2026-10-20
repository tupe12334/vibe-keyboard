//! The real `PluginHost`: plugin side effects on the live deck, TUI and desktop apps.

use std::sync::{Arc, Mutex};

use super::{open_claude_terminal, open_config_in_vscode, open_in_chrome, open_log_file};
use crate::application::pages::{fetch_centy_projects, moadim};
use crate::application::{lock, render_screen};
use crate::domain::actions::{CentyProject, ClaudeSession};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::claude_sessions::list_sessions;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::plugins::PluginHost;
use crate::presentation::tui;

pub(super) struct DeckHost<'a> {
    pub nav: &'a mut NavigationStack,
    pub handle: &'a Device,
    pub state: &'a Arc<Mutex<tui::AppState>>,
    pub dev_state: &'a Arc<Mutex<DeviceState>>,
}

impl PluginHost for DeckHost<'_> {
    fn open_terminal(&mut self) {
        super::open_terminal();
    }
    fn open_claude_terminal(&mut self) {
        open_claude_terminal();
    }
    fn open_in_chrome(&mut self, url: &str) {
        open_in_chrome(url);
    }
    fn open_log_file(&mut self) {
        open_log_file();
    }
    fn open_config_in_vscode(&mut self) {
        open_config_in_vscode();
    }
    fn open_routines(&mut self) {
        moadim::open_routines(self.nav, self.handle, self.state, self.dev_state);
    }
    fn fetch_centy_projects(&mut self) -> Vec<CentyProject> {
        fetch_centy_projects()
    }
    fn list_sessions(&mut self) -> Vec<ClaudeSession> {
        list_sessions()
    }
    fn set_loading(&mut self, loading: bool) {
        lock(self.state).loading = loading;
    }
    fn show(&mut self, screen: Screen) {
        self.nav.push(screen);
        render_screen(self.nav, self.handle, self.state, self.dev_state);
    }
}
