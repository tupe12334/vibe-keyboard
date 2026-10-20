use std::sync::{Arc, Mutex};

use super::pages::{moadim::MoadimRefresh, SessionsRefresh};
use crate::domain::navigation::NavigationStack;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

#[derive(Default)]
pub struct Refresh {
    pub sessions: SessionsRefresh,
    pub moadim: MoadimRefresh,
}

impl Refresh {
    pub fn tick(
        &mut self,
        nav: &mut NavigationStack,
        handle: &Device,
        state: &Arc<Mutex<tui::AppState>>,
        dev_state: &Arc<Mutex<DeviceState>>,
    ) {
        self.sessions.tick(nav, handle, state, dev_state);
        self.moadim.tick(nav, handle, state, dev_state);
    }
}
