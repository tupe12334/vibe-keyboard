//! Page lists fetched on a background thread, so the device loop never waits on a subprocess.

use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex, PoisonError};

use tracing::info;

use crate::application::render_screen;
use crate::domain::actions::{CentyProject, ClaudeSession};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// A page's list, fetched on a background thread.
pub(super) enum Fetched {
    Projects(Vec<CentyProject>),
    Sessions(Vec<ClaudeSession>),
}

/// The fetch in flight. One at a time: `loading` ignores keys until it lands.
static PENDING: Mutex<Option<Receiver<Fetched>>> = Mutex::new(None);

/// Run `fetch` on a thread and set `loading`. `poll_pending` shows the result. Never blocks.
pub(super) fn start_fetch(
    state: &Arc<Mutex<tui::AppState>>,
    fetch: impl FnOnce() -> Fetched + Send + 'static,
) {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _ = tx.send(fetch());
    });
    *PENDING.lock().unwrap_or_else(PoisonError::into_inner) = Some(rx);
    crate::application::lock(state).loading = true;
}

/// Show the page once its fetch lands. Call once per device-loop pass. Never blocks.
pub fn poll_pending(
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let mut pending = PENDING.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(rx) = pending.as_ref() else {
        return;
    };
    let fetched = match rx.try_recv() {
        Ok(fetched) => Some(fetched),
        Err(TryRecvError::Empty) => return,
        // The fetch thread panicked. Clear loading and show nothing.
        Err(TryRecvError::Disconnected) => None,
    };
    *pending = None;
    drop(pending);
    crate::application::lock(state).loading = false;
    let Some(screen) = fetched.and_then(next_screen) else {
        return;
    };
    nav.push(screen);
    render_screen(nav, handle, state, dev_state);
}

/// The page to show for a fetch. None keeps the current page (no projects found).
fn next_screen(fetched: Fetched) -> Option<Screen> {
    match fetched {
        Fetched::Projects(projects) if projects.is_empty() => None,
        Fetched::Projects(projects) => Some(Screen::CentyProjectList {
            projects,
            page: 0,
            filter: None,
        }),
        Fetched::Sessions(sessions) => {
            info!("claude sessions: {} found", sessions.len());
            Some(Screen::ClaudeSessions { sessions, page: 0 })
        }
    }
}

#[cfg(test)]
#[path = "background_fetch_tests.rs"]
mod tests;
