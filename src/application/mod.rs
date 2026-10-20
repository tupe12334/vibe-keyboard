mod actions;
mod centy_actions;
mod moadim_keys;
mod pages;
pub mod prompt_return;
mod session_actions;
mod voice;

pub use pages::{refresh_side_widgets, render_screen};
pub use refresh::Refresh;

mod refresh;

mod dictation;
mod frontmost;
mod gestures;
mod holds;
mod input_modes;

pub use frontmost::handle_frontmost;
pub use holds::{install_panic_release, poll_holds, read_wait, release_all_holds};
pub use voice::expire_flash;

use crate::infrastructure::usb::Device;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::presentation::tui;

fn lock(state: &Arc<Mutex<tui::AppState>>) -> MutexGuard<'_, tui::AppState> {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn handle_action_key(
    key: u8,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    // Extract what we need from the current screen before any mutable nav operations.
    // Main page keys come from the user's profile, not from code.
    if let Screen::MainPage { page } = nav.current() {
        let page = *page;
        // A key with `hold` waits for its gesture. `end_press` runs the tap.
        let configured = lock(state)
            .profile
            .key_at(page, key)
            .filter(|config| config.hold.is_none())
            .map(|config| config.action.clone());
        if let Some(action) = configured {
            actions::run_action(&action, nav, handle, state, dev_state);
        }
        return;
    }

    let action = action_for_key(key, nav, state);

    match action {
        Action::None => {}
        action @ (Action::SelectProject { .. }
        | Action::OpenVsCode { .. }
        | Action::OpenTerminalInPath { .. }
        | Action::OpenBrowser { .. }
        | Action::FetchIssues { .. }
        | Action::SelectIssue { .. }
        | Action::OpenIssueInVsCode { .. }
        | Action::OpenIssueInWeb { .. }
        | Action::OpenCentyWorkspace { .. }) => {
            centy_actions::centy_action(action, nav, handle, state, dev_state);
        }
        action => session_actions::session_action(action, nav, handle, state, dev_state),
    }
}

mod filter_query;
mod key_action;
use key_action::{action_for_key, Action};
mod key_event;
pub use filter_query::handle_filter_query;
pub use key_event::handle_key_event;
