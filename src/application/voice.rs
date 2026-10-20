//! The voice key (13) on a session's action page. A hold pushes to talk (`dictation.rs`). A tap
//! sends Enter to that session, so a finished dictation goes out.

use std::sync::{Arc, Mutex};

use super::key_action::Action;
use super::session_actions::session_action;
use crate::domain::actions::TerminalApp;
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// The key on a session action page that holds voice.
pub const VOICE_KEY: u8 = 13;

/// Whether a press of `key` on this screen is the voice hold. Apps that take no text are muted.
pub fn holds_voice(screen: &Screen, key: u8) -> bool {
    matches!(screen, Screen::SessionActions { session, .. }
        if key == VOICE_KEY && session.app != TerminalApp::Other)
}

/// The send a tap on the voice key makes: Enter alone, to the session on screen. None when the
/// screen is not a session that takes text. Empty text types nothing but Enter.
pub fn enter_action(screen: &Screen) -> Option<Action> {
    match screen {
        Screen::SessionActions { session, .. } if holds_voice(screen, VOICE_KEY) => {
            Some(Action::SendPrompt {
                pid: session.pid,
                tty: session.tty.clone(),
                text: String::new(),
            })
        }
        _ => None,
    }
}

/// A tap on the voice key: send Enter to the session on screen, through the prompt send path.
pub fn tap(
    handle: &Device,
    nav: &mut NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    if let Some(action) = enter_action(nav.current()) {
        session_action(action, nav, handle, state, dev_state);
    }
}

#[cfg(test)]
#[path = "voice_tests.rs"]
mod tests;
