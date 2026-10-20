//! Device-loop side of the holds: reads the next wait, starts due holds, and ends them on
//! release, page change, disconnect and shutdown.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::domain::hold::{Held, HoldTracker};
use crate::domain::keys::is_nav_key;
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::keyboard;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

use super::gestures::run_gestures;

/// Longest wait for a device event. Shorter while a key is due to become a hold.
pub const READ_TIMEOUT: Duration = Duration::from_millis(500);

/// How long the device read may block: `READ_TIMEOUT`, or less when a key will become a hold first.
pub fn read_wait(state: &Arc<Mutex<tui::AppState>>) -> Duration {
    let s = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let threshold = s.profile.hold_threshold();
    s.holds
        .next_deadline(Instant::now(), threshold)
        .map_or(READ_TIMEOUT, |wait| wait.min(READ_TIMEOUT))
}

/// Start the holds that are due now. Called from the device loop after each read.
pub fn poll_holds(
    handle: &Device,
    nav: &mut NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    keyboard::hold_touch();
    let gestures = {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let threshold = s.profile.hold_threshold();
        s.holds.poll(Instant::now(), threshold)
    };
    run_gestures(gestures, handle, nav, state, dev_state);
}

/// Stop every hold now and forget the keys that are down. Their releases then do nothing.
/// Called on shutdown, device disconnect and page change, so no key stays down.
pub fn release_all_holds(state: &Arc<Mutex<tui::AppState>>) {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .holds = HoldTracker::default();
    keyboard::release_all();
}

/// Lift a held key if the process panics. Must run before the panic unwinds.
pub fn install_panic_release() {
    keyboard::install_panic_release();
}

/// Remember a press whose action waits for the hold decision. Covers main page keys with `hold`,
/// and session keys (focus still runs at press, the hold adds dictation).
pub(super) fn track_press(key: u8, nav: &NavigationStack, state: &Arc<Mutex<tui::AppState>>) {
    if is_nav_key(key) {
        return;
    }
    let held = match nav.current() {
        Screen::MainPage { page } => state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .profile
            .key_at(*page, key)
            .and_then(|config| {
                config.hold.clone().map(|hold| Held::Action {
                    short: config.action.clone(),
                    hold: Some(hold),
                })
            }),
        Screen::ClaudeSessions { sessions, page } if matches!(key, 1..=10) => sessions
            .get(page * 10 + (key as usize - 1))
            .map(|session| Held::Session {
                tty: session.tty.clone(),
            }),
        _ => None,
    };
    if let Some(held) = held {
        state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .holds
            .press(key, Instant::now(), held);
    }
}
