use std::sync::{Arc, Mutex};

use tracing::info;

use super::render_screen;
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

#[path = "filter_match.rs"]
mod filter_match;

pub fn handle_filter_query(
    query: &str,
    handle: &Device,
    nav: &mut NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    use filter_match::FilterAction;

    match filter_match::filter_action(nav, query) {
        FilterAction::Projects { projects, filter } => {
            info!(
                "filter applied: {} project(s) match {:?}",
                projects.len(),
                query
            );
            nav.push(Screen::CentyProjectList {
                projects,
                page: 0,
                filter,
            });
            render_screen(nav, handle, state, dev_state);
        }
        FilterAction::Issues {
            issues,
            project_name,
            org,
            filter,
        } => {
            info!(
                "filter applied: {} issue(s) match {:?}",
                issues.len(),
                query
            );
            nav.push(Screen::CentyIssueList {
                issues,
                page: 0,
                project_name,
                org,
                filter,
            });
            render_screen(nav, handle, state, dev_state);
        }
        FilterAction::None => {}
    }
}
