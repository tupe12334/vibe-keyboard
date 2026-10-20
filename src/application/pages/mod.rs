mod centy;
mod claude_sessions;
mod input_number;
mod issue_actions;
mod issue_list;
pub mod key_tiles;
mod main_page;
pub mod moadim;
mod new_session;
mod project_actions;
mod project_list;
mod session_actions;

pub use centy::{fetch_centy_issues, fetch_centy_projects};
pub use claude_sessions::SessionsRefresh;
pub use key_tiles::show_focus_only;
pub use main_page::refresh_side_widgets;
use main_page::render_side;

use crate::infrastructure::usb::Device;
use std::sync::{Arc, Mutex};

use crate::domain::navigation::{back_key_on, BackKey, NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::clear_all;
use crate::presentation::tui;

/// Render the given screen to both the TUI state and the hardware device.
pub fn render_screen(
    nav: &NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    {
        let mut s = super::lock(state);
        s.nav_can_back = NavigationStack::can_back();
        s.nav_can_out = nav.can_out();
        s.nav_can_forward = nav.can_forward();
        // clear_all below blanks the side screen too, so the sent frame is no longer shown.
        s.side_frame = None;
    }
    clear_all(handle);
    let screen = nav.current();
    match screen {
        Screen::MainPage { page } => {
            main_page::render_main_page(*page, handle, state, dev_state);
        }
        Screen::CentyProjectList {
            projects,
            page,
            filter,
        } => {
            project_list::render_project_list(projects, *page, filter.as_deref(), handle, state);
        }
        Screen::CentyProjectActions { project } => {
            project_actions::render_project_actions(project, handle, state);
        }
        Screen::CentyIssueList {
            issues,
            page,
            project_name,
            filter,
            ..
        } => {
            issue_list::render_issue_list(
                issues,
                *page,
                project_name,
                filter.as_deref(),
                handle,
                state,
            );
        }
        Screen::CentyIssueActions {
            issue,
            project_name,
            org,
        } => {
            issue_actions::render_issue_actions(issue, project_name, org, handle, state);
        }
        Screen::InputNumber { value } => {
            input_number::render_input_number(value, handle, state);
        }
        Screen::ClaudeSessions { sessions, page } => {
            claude_sessions::render_claude_sessions(sessions, *page, handle, state);
        }
        Screen::NewSessionChoice => new_session::render_new_session_choice(handle, state),
        Screen::SessionActions { session, page } => {
            session_actions::render_session_actions(session, *page, handle, state);
        }
        _ => moadim::render(screen, handle, state),
    }
    // Every sub-screen shows OUT on key 11. Drawn last, so a page cannot cover it.
    if back_key_on(screen) == BackKey::Out {
        key_tiles::show_out(handle, state);
    }
    render_side(handle, state, nav);
}
