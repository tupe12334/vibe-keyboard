//! Built-in actions: named shortcuts a profile binds to a key, such as Terminal or the numpad.

use std::sync::{Arc, Mutex};

use tracing::{error, info};

use super::{
    open_claude_terminal, open_config_in_vscode, open_in_chrome, open_log_file, open_terminal,
};
use crate::application::pages::fetch_centy_projects;
use crate::application::render_screen;
use crate::domain::actions::CentyProject;
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::claude_sessions::list_sessions;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

pub(super) fn run_builtin(
    name: &str,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    match name {
        "centy_projects" => {
            crate::application::lock(state).loading = true;
            let projects: Vec<CentyProject> = fetch_centy_projects();
            crate::application::lock(state).loading = false;
            if !projects.is_empty() {
                nav.push(Screen::CentyProjectList {
                    projects,
                    page: 0,
                    filter: None,
                });
                render_screen(nav, handle, state, dev_state);
            }
        }
        "terminal" => {
            info!("opening Terminal");
            open_terminal();
        }
        "claude" => {
            info!("opening Claude in Terminal");
            open_claude_terminal();
        }
        "centy_web" => {
            info!("opening Centy in Chrome");
            open_in_chrome("https://app.centy.io");
        }
        "numpad" => {
            info!("opening input number page");
            nav.push(Screen::InputNumber {
                value: String::new(),
            });
            render_screen(nav, handle, state, dev_state);
        }
        "log_file" => {
            info!("opening log file in VS Code");
            open_log_file();
        }
        "vscode_config" => {
            info!("opening config in VS Code");
            open_config_in_vscode();
        }
        "claude_sessions" => {
            crate::application::lock(state).loading = true;
            let sessions = list_sessions();
            crate::application::lock(state).loading = false;
            info!("claude sessions: {} found", sessions.len());
            nav.push(Screen::ClaudeSessions { sessions, page: 0 });
            render_screen(nav, handle, state, dev_state);
        }
        "moadim" => crate::application::pages::moadim::open_routines(nav, handle, state, dev_state),
        other => error!("profile: unknown built-in '{other}'"),
    }
}
