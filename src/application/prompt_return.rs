//! A prompt that reaches its session takes the user back to the sessions list. The send runs off
//! the device loop, so its result comes back on a channel that `poll_sent_prompt` reads each turn.

use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex, PoisonError};

use super::render_screen;
use crate::domain::actions::TerminalApp;
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// The send still waiting: its tty, and whether it reached the session.
static PENDING: Mutex<Option<Receiver<(String, bool)>>> = Mutex::new(None);

/// Muted apps (`TerminalApp::Other`) take no typed text. Their prompt keys only focus the session.
pub const fn may_send(app: TerminalApp) -> bool {
    !matches!(app, TerminalApp::Other)
}

/// Back to the sessions list. Key 12 and a prompt that reached its session both come here.
pub fn back_to_sessions(nav: &mut NavigationStack) {
    // The exit confirm sits above its session page, so leave that first.
    if matches!(nav.current(), Screen::ConfirmExit { .. }) {
        nav.out();
    }
    nav.back();
}

/// Runs `send` on a thread. Its result reaches `poll_sent_prompt` with `tty`. A send that is still
/// waiting is dropped, so the newest send wins. ponytail: one send at a time, add a queue if needed.
pub fn start_prompt(tty: String, send: impl FnOnce() -> bool + Send + 'static) {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _ = tx.send((tty, send()));
    });
    *PENDING.lock().unwrap_or_else(PoisonError::into_inner) = Some(rx);
}

/// Called once per loop turn. Leaves the action page when the last prompt reached its session.
pub fn poll_sent_prompt(
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let Some((tty, sent)) = take_result() else {
        return;
    };
    if leave_after_send(nav, &tty, sent) {
        back_to_sessions(nav);
        render_screen(nav, handle, state, dev_state);
    }
}

/// The waiting result, if it has arrived. Clears the slot once the result is in or the sender is gone.
fn take_result() -> Option<(String, bool)> {
    let mut pending = PENDING.lock().unwrap_or_else(PoisonError::into_inner);
    let result = pending.as_ref()?.try_recv();
    if !matches!(result, Err(TryRecvError::Empty)) {
        *pending = None;
    }
    result.ok()
}

/// True when a delivered send for `tty` should leave the page. Only that session's action page
/// counts, so a user who already left stays where they are.
pub fn leave_after_send(nav: &NavigationStack, tty: &str, sent: bool) -> bool {
    sent && matches!(
        nav.current(),
        Screen::SessionActions { session, .. } | Screen::ConfirmExit { session } if session.tty == tty
    )
}

#[cfg(test)]
#[path = "prompt_return_tests.rs"]
mod tests;
