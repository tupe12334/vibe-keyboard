//! A device-less app driven through the real key handler and hold poll.

use std::sync::{Arc, Mutex};

use super::recorder::{clear, FAKE};
use crate::application::actions::effects::{install, Restore};
use crate::application::{handle_key_event, lock, poll_holds, Refresh};
use crate::domain::navigation::NavigationStack;
use crate::domain::profile::{Action, KeyConfig, Page, Profile};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui::AppState;

/// Raw deck id of logical key 1 (see `domain/keys.rs`).
const KEY_ONE_RAW: u8 = 0x0D;
/// Long enough that a press never becomes a hold within a test.
const NEVER_HOLD_MS: u64 = 60_000;

/// A device-less app with two pages. Key 1 on page 1 has `action` and an optional `hold`.
pub struct Rig {
    pub nav: NavigationStack,
    pub state: Arc<Mutex<AppState>>,
    pub dev_state: Arc<Mutex<DeviceState>>,
    device: Device,
    refresh: Refresh,
    _effects: Restore,
}

impl Rig {
    /// A press runs `action`. A press with a `hold` becomes a hold at once.
    pub fn new(action: Action, hold: Option<Action>) -> Self {
        Self::with_hold_ms(action, hold, 0)
    }

    /// Like `new`, but a press only becomes a hold after a minute, so a release is a tap.
    pub fn slow(action: Action, hold: Option<Action>) -> Self {
        Self::with_hold_ms(action, hold, NEVER_HOLD_MS)
    }

    fn with_hold_ms(action: Action, hold: Option<Action>, hold_ms: u64) -> Self {
        clear();
        let mut profile = Profile {
            pages: vec![Page::new("one"), Page::new("two")],
            ..Profile::default()
        };
        let config = KeyConfig {
            action,
            hold,
            ..KeyConfig::default()
        };
        profile.set_key(0, 1, config);
        profile.input.hold_ms = hold_ms;
        let state = Arc::new(Mutex::new(AppState::new(2)));
        lock(&state).profile = profile;
        Self {
            nav: NavigationStack::new(0, 2),
            state,
            dev_state: Arc::new(Mutex::new(DeviceState {
                current_page: 0,
                brightness: 0,
            })),
            device: Device::detached(),
            refresh: Refresh::default(),
            _effects: install(FAKE),
        }
    }

    /// Key 1 down, through the same handler the device loop uses.
    pub fn press(&mut self) {
        self.event(1);
    }

    /// Key 1 up.
    pub fn release(&mut self) {
        self.event(0);
    }

    /// One device-loop pass that starts every hold that is due.
    pub fn poll(&mut self) {
        poll_holds(&self.device, &mut self.nav, &self.state, &self.dev_state);
    }

    fn event(&mut self, down: u8) {
        let mut data = vec![0; 11];
        data[9] = KEY_ONE_RAW;
        data[10] = down;
        handle_key_event(
            &data,
            &self.device,
            &mut self.nav,
            &self.state,
            &self.dev_state,
            &mut self.refresh,
        );
    }
}
