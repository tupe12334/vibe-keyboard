mod actions;
mod moadim_keys;
mod pages;
pub mod prompt_return;
mod voice;

pub use pages::{refresh_side_widgets, render_screen};
pub use refresh::Refresh;

mod refresh;

mod dictation;
mod frontmost;
mod gestures;
mod holds;
mod input_modes;
use input_modes::NumberEdit;

pub use frontmost::handle_frontmost;
pub use holds::{install_panic_release, poll_holds, read_wait, release_all_holds};
pub use voice::expire_flash;

use crate::infrastructure::claude_sessions::{focus_session, open_new_session, send_prompt};
use crate::infrastructure::usb::Device;
use actions::{open_centy_workspace, open_in_chrome, open_terminal_in_path, open_vscode_in_path};
use pages::{fetch_centy_issues, open_session_actions, show_focus_only};
use std::sync::{Arc, Mutex, MutexGuard};
use tracing::info;

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
        Action::SelectProject { projects, idx } => {
            if let Some(project) = projects.into_iter().nth(idx) {
                info!("centy: selected project {}", project.name);
                nav.push(Screen::CentyProjectActions { project });
                render_screen(nav, handle, state, dev_state);
            }
        }
        Action::OpenVsCode { name, path } => {
            info!("centy: open {} in VS Code", name);
            open_vscode_in_path(path.as_deref().unwrap_or("."));
        }
        Action::OpenTerminalInPath { name, path } => {
            info!("centy: open {} in Terminal", name);
            open_terminal_in_path(path.as_deref());
        }
        Action::OpenBrowser { name, url } => {
            info!("centy: open {} in browser", name);
            open_in_chrome(&url);
        }
        Action::FetchIssues { project_name, org } => {
            lock(state).loading = true;
            let issues = fetch_centy_issues(&project_name);
            lock(state).loading = false;
            if !issues.is_empty() {
                nav.push(Screen::CentyIssueList {
                    issues,
                    page: 0,
                    project_name,
                    org,
                    filter: None,
                });
                render_screen(nav, handle, state, dev_state);
            }
        }
        Action::SelectIssue {
            issues,
            project_name,
            org,
            idx,
        } => {
            if let Some(issue) = issues.into_iter().nth(idx) {
                info!("centy: selected issue #{}", issue.number);
                nav.push(Screen::CentyIssueActions {
                    issue,
                    project_name,
                    org,
                });
                render_screen(nav, handle, state, dev_state);
            }
        }
        Action::OpenIssueInVsCode { file_path } => {
            let path = file_path.as_deref().unwrap_or(".");
            info!("centy: open issue in VS Code at {}", path);
            open_vscode_in_path(path);
        }
        Action::OpenIssueInWeb {
            id,
            org,
            project_name,
        } => {
            let url = format!("https://app.centy.io/{org}/{project_name}/issues/{id}");
            info!("centy: open issue in web: {}", url);
            open_in_chrome(&url);
        }
        Action::OpenCentyWorkspace { issue_number } => {
            info!("centy: opening workspace for issue {}", issue_number);
            open_centy_workspace(issue_number);
        }
        Action::SortList => {
            info!("sort: toggling sort order");
            nav.toggle_sort();
            render_screen(nav, handle, state, dev_state);
        }
        Action::OpenSessionActions { session } => {
            info!("claude sessions: actions for {}", session.tty);
            // Selecting a session focuses its tab first, then opens the action page.
            open_session_actions(nav, session, |tty| {
                let tty = tty.to_string();
                // Off the device loop: osascript can take a while to answer.
                std::thread::spawn(move || focus_session(&tty));
            });
            render_screen(nav, handle, state, dev_state);
        }
        Action::SendPrompt { pid, tty, text } => {
            info!("claude sessions: send prompt to {tty}");
            // Off the device loop: osascript can take a while. Leaves the page once it is sent.
            prompt_return::start_prompt(tty.clone(), move || send_prompt(pid, &tty, &text));
        }
        Action::TurnActionPage { forward, count } => {
            nav.turn_action_page(forward, count);
            render_screen(nav, handle, state, dev_state);
        }
        Action::PromptUnsupported { key } => {
            info!("claude sessions: key {key} needs iTerm2 or Terminal; focus only");
            show_focus_only(handle, key, state);
        }
        Action::OpenNewSessionChoice => {
            info!("claude sessions: choose a terminal for a new session");
            nav.push(Screen::NewSessionChoice);
            render_screen(nav, handle, state, dev_state);
        }
        Action::NewSession { terminal } => {
            info!("claude sessions: new session in {terminal:?}");
            // Off the device loop: osascript can take a while to answer.
            std::thread::spawn(move || open_new_session(terminal));
            // Back to the list. The new session shows on the next refresh.
            nav.back();
            render_screen(nav, handle, state, dev_state);
        }
        Action::TurnSessionsPage { forward } => {
            if forward {
                nav.forward();
            } else {
                nav.previous_page();
            }
            render_screen(nav, handle, state, dev_state);
        }
        Action::InputNumberDigit { digit } => {
            input_modes::edit_number(NumberEdit::Digit(digit), nav, handle, state, dev_state);
        }
        Action::InputNumberClear => {
            input_modes::edit_number(NumberEdit::Clear, nav, handle, state, dev_state);
        }
        Action::InputNumberBackspace => {
            input_modes::edit_number(NumberEdit::Backspace, nav, handle, state, dev_state);
        }
        Action::Search => input_modes::start_filter_input(state),
        Action::None => {}
    }
}

mod filter_query;
mod key_action;
use key_action::{action_for_key, Action};
mod key_event;
pub use filter_query::handle_filter_query;
pub use key_event::handle_key_event;
