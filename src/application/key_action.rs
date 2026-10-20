use std::sync::{Arc, Mutex};

pub(super) use action::Action;

use crate::domain::navigation::{NavigationStack, Screen};
use crate::presentation::tui;

mod action;
mod centy;
mod input_number;
mod sessions;

/// Decides what the key press does on the current screen. Main pages never reach this:
/// their keys come from the profile and are handled first.
pub(super) fn action_for_key(
    key: u8,
    nav: &NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
) -> Action {
    let screen = nav.current();
    match screen {
        // Moadim pages are handled by `moadim_keys` before this match.
        Screen::MainPage { .. } | Screen::MoadimList { .. } | Screen::MoadimRoutine { .. } => {
            Action::None
        }
        Screen::CentyProjectList { .. }
        | Screen::CentyProjectActions { .. }
        | Screen::CentyIssueList { .. }
        | Screen::CentyIssueActions { .. } => centy::action(key, screen),
        Screen::ClaudeSessions { .. }
        | Screen::SessionActions { .. }
        | Screen::ConfirmExit { .. }
        | Screen::NewSessionChoice => sessions::action(key, screen, state),
        Screen::InputNumber { .. } => input_number::action(key, screen),
    }
}
