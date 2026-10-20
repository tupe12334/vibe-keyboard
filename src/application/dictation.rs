//! Starts and ends the dictation of a hold: push-to-talk keys, and Claude sessions held to focus.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tracing::{error, info};

use super::actions;
use crate::domain::hotkey::{default_push_to_talk, push_to_talk_key};
use crate::domain::navigation::NavigationStack;
use crate::infrastructure::claude_sessions::focus_session;
use crate::infrastructure::keyboard;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// Time a session key waits after focus before dictation starts. The terminal needs it to take focus.
const FOCUS_SETTLE: Duration = Duration::from_millis(150);

/// A hold reached the threshold. Push-to-talk presses its key down; other actions run now.
pub(super) fn start_hold(
    key: u8,
    hold: &crate::domain::profile::Action,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    if let crate::domain::profile::Action::PushToTalk { key: binding } = hold {
        match push_to_talk_key(binding.as_deref()) {
            Ok(hotkey) => {
                info!("profile: push-to-talk hold start");
                let generation = keyboard::hold_begin(key);
                keyboard::push_to_talk_start(generation, key, &hotkey);
            }
            Err(e) => tracing::error!("profile: bad push-to-talk key: {e}"),
        }
    } else {
        actions::run_action(hold, nav, handle, state, dev_state);
    }
}

/// A hold ended. Only push-to-talk has work to do: key up.
pub(super) fn end_hold(key: u8, hold: &crate::domain::profile::Action) {
    if matches!(hold, crate::domain::profile::Action::PushToTalk { .. }) {
        keyboard::hold_end(key);
    }
}

/// Session key held: focus its tab, let the terminal settle, then start dictation.
/// Runs off the device loop, because focusing waits for osascript.
pub(super) fn start_session_dictation(tty: String, key: u8) {
    let generation = keyboard::hold_begin(key);
    info!("claude sessions: dictate in {tty}");
    thread::spawn(move || {
        // No terminal tab: Space would reach the wrong app, so dictation does not start.
        if !focus_session(&tty) {
            error!("claude sessions: NO FOCUS for {tty}, dictation not started");
            return;
        }
        thread::sleep(FOCUS_SETTLE);
        keyboard::push_to_talk_start(generation, key, &default_push_to_talk());
    });
}
