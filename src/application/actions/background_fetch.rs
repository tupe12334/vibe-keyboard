//! Page lists fetched on a background thread, so the device loop never waits on a subprocess.

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex, PoisonError};

use super::fetch_result::{take_ready, Fetched, Pending, Ready};
use crate::application::render_screen;
use crate::domain::navigation::{NavigationStack, ScreenId};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// The fetch in flight. One at a time: `loading` ignores keys until it lands.
static PENDING: Mutex<Option<Pending>> = Mutex::new(None);

/// Run `fetch` on a thread and set `loading`. `poll_pending` shows the result. Never blocks.
pub(super) fn start_fetch(
    state: &Arc<Mutex<tui::AppState>>,
    from: ScreenId,
    fetch: impl FnOnce() -> Fetched + Send + 'static,
) {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let _ = tx.send(fetch());
    });
    *PENDING.lock().unwrap_or_else(PoisonError::into_inner) = Some(Pending { rx, from });
    crate::application::lock(state).loading = true;
}

/// Show the page once its fetch lands. Call on every device-loop pass. Never blocks.
pub fn poll_pending(
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let ready = take_ready(
        &mut PENDING.lock().unwrap_or_else(PoisonError::into_inner),
        nav.screen_id(),
    );
    let screen = match ready {
        Ready::Idle | Ready::Waiting => return,
        Ready::Done(screen) => screen,
    };
    crate::application::lock(state).loading = false;
    if let Some(screen) = screen {
        nav.push(screen);
        render_screen(nav, handle, state, dev_state);
    }
}
