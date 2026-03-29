mod centy;
mod input_number;
mod issue_actions;
mod issue_list;
mod main_page;
mod project_actions;
mod project_list;

pub use centy::{fetch_centy_issues, fetch_centy_projects};

use rusb::{Context, DeviceHandle};
use std::sync::{Arc, Mutex};

use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::clear_all;
use crate::presentation::tui;

/// Render the given screen to both the TUI state and the hardware device.
pub fn render_screen(
    nav: &NavigationStack,
    handle: &DeviceHandle<Context>,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    {
        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
        s.nav_can_back = nav.can_back();
        s.nav_can_out = nav.can_out();
        s.nav_can_forward = nav.can_forward();
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
    }
}
