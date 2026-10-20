//! Turns each gesture of a key into its action: a tap, the start of a hold, or the end of one.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::domain::hold::{Gesture, Held};
use crate::domain::navigation::NavigationStack;
use crate::infrastructure::keyboard;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

use super::actions;
use super::dictation::{end_hold, start_hold, start_session_dictation, start_voice};
use super::voice;

/// Key up: finish the press. A short press runs its tap, a long one ends its hold.
pub(super) fn end_press(
    key: u8,
    handle: &Device,
    nav: &mut NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let gestures = {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let threshold = s.profile.hold_threshold();
        s.holds.release(key, Instant::now(), threshold)
    };
    run_gestures(gestures, handle, nav, state, dev_state);
}

pub(super) fn run_gestures(
    gestures: Vec<Gesture>,
    handle: &Device,
    nav: &mut NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    for gesture in gestures {
        match gesture {
            Gesture::Tap {
                held: Held::Action { short, .. },
                ..
            } => actions::run_action(&short, nav, handle, state, dev_state),
            Gesture::HoldStart {
                key,
                held: Held::Action {
                    hold: Some(hold), ..
                },
            } => start_hold(key, &hold, nav, handle, state, dev_state),
            Gesture::HoldEnd {
                key,
                held: Held::Action {
                    hold: Some(hold), ..
                },
            } => end_hold(key, &hold),
            Gesture::HoldStart {
                key,
                held: Held::Session { tty },
            } => start_session_dictation(tty, key),
            Gesture::HoldEnd {
                key,
                held: Held::Session { .. } | Held::Voice,
            } => keyboard::hold_end(key),
            Gesture::HoldStart {
                key,
                held: Held::Voice,
            } => start_voice(key, nav),
            Gesture::Tap {
                held: Held::Voice, ..
            } => voice::flash(handle, state),
            _ => {}
        }
    }
}
