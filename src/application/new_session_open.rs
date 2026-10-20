//! A new session opens its action page on the deck, so the user can hold Voice and dictate at once.
//! The terminal opens off the device loop. The new session comes back on a channel that
//! `poll_new_session` reads each turn.

use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use tracing::{error, info};

use super::pages::open_session_actions;
use super::render_screen;
use crate::domain::actions::{ClaudeSession, NewSessionTerminal};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::claude_sessions::{focus_session, list_sessions, open_new_session};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// How long the new `claude` process has to show up in the session list.
const APPEAR_TIMEOUT: Duration = Duration::from_secs(20);
const APPEAR_POLL: Duration = Duration::from_millis(500);

/// The new session still waiting to appear.
static PENDING: Mutex<Option<Receiver<ClaudeSession>>> = Mutex::new(None);

/// Opens a new session in `terminal` on a thread, then waits for its `claude` process.
/// ponytail: one new session at a time, the newest wins.
pub fn start_new_session(terminal: NewSessionTerminal) {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let before: Vec<String> = list_sessions().into_iter().map(|s| s.tty).collect();
        open_new_session(terminal);
        let deadline = Instant::now() + APPEAR_TIMEOUT;
        while Instant::now() < deadline {
            if let Some(session) = newest(list_sessions(), &before) {
                let _ = tx.send(session);
                return;
            }
            std::thread::sleep(APPEAR_POLL);
        }
        error!("claude sessions: new session did not appear in {APPEAR_TIMEOUT:?}");
    });
    *PENDING.lock().unwrap_or_else(PoisonError::into_inner) = Some(rx);
}

/// The first session on a tty that was not there before.
fn newest(now: Vec<ClaudeSession>, before: &[String]) -> Option<ClaudeSession> {
    now.into_iter().find(|s| !before.contains(&s.tty))
}

/// Called once per loop turn. Opens the new session's action page when it has appeared and the
/// user is still on the sessions list.
pub fn poll_new_session(
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let Some(session) = take_result() else {
        return;
    };
    if !opens_on(nav) {
        return;
    }
    info!(
        "claude sessions: new session {} ready, voice on key 13",
        session.tty
    );
    open_session_actions(nav, session, |tty| {
        let tty = tty.to_string();
        std::thread::spawn(move || focus_session(&tty));
    });
    render_screen(nav, handle, state, dev_state);
}

/// True while the deck shows the sessions list. A user who moved on stays where they are.
fn opens_on(nav: &NavigationStack) -> bool {
    matches!(nav.current(), Screen::ClaudeSessions { .. })
}

fn take_result() -> Option<ClaudeSession> {
    let mut pending = PENDING.lock().unwrap_or_else(PoisonError::into_inner);
    let result = pending.as_ref()?.try_recv();
    if !matches!(result, Err(TryRecvError::Empty)) {
        *pending = None;
    }
    result.ok()
}

#[cfg(test)]
#[path = "new_session_open_tests.rs"]
mod tests;
