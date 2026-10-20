use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use tracing::{error, info};

use super::{
    open_claude_terminal, open_config_in_vscode, open_in_chrome, open_log_file, open_terminal,
};
use crate::application::pages::fetch_centy_projects;
use crate::application::render_screen;
use crate::domain::actions::CentyProject;
use crate::domain::hotkey::{parse_hotkey, push_to_talk_key};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::domain::profile::{is_bundle_id, Action};
use crate::infrastructure::claude_sessions::list_sessions;
use crate::infrastructure::keyboard;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// Run the action bound to a main page key.
pub fn run_action(
    action: &Action,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    match action {
        Action::None => {}
        Action::OpenUrl { url } => {
            info!("profile: open url {url}");
            spawn_detached("open", &["--", url.as_str()]);
        }
        Action::RunCommand { command } => {
            info!("profile: run command");
            spawn_detached("sh", &["-c", command.as_str()]);
        }
        Action::OpenApp { name } => {
            info!("profile: open app {name}");
            if is_bundle_id(name) {
                spawn_detached("open", &["-b", name.as_str()]);
            } else {
                spawn_detached("open", &["-a", name.as_str()]);
            }
        }
        Action::Hotkey { keys } => match parse_hotkey(keys) {
            Ok(hotkey) => {
                info!("profile: hotkey {keys}");
                let script = hotkey.applescript();
                spawn_detached("osascript", &["-e", script.as_str()]);
            }
            Err(e) => error!("profile: bad hotkey '{keys}': {e}"),
        },
        Action::PushToTalk { key } => match push_to_talk_key(key.as_deref()) {
            Ok(hotkey) => {
                info!("profile: push-to-talk tap");
                keyboard::tap(&hotkey);
            }
            Err(e) => error!("profile: bad push-to-talk key: {e}"),
        },
        Action::GoToPage { page } => {
            crate::application::lock(state).auto_page.on_manual_page();
            nav.go_to_page(*page);
            render_screen(nav, handle, state, dev_state);
        }
        Action::Builtin { name } => run_builtin(name, nav, handle, state, dev_state),
    }
}

fn run_builtin(
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

/// Start a program without waiting for it. A background thread reaps the child.
fn spawn_detached(program: &str, args: &[&str]) {
    match Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            thread::spawn(move || {
                if let Err(e) = child.wait() {
                    error!("wait failed: {e}");
                }
            });
        }
        Err(e) => error!("failed to start {program}: {e}"),
    }
}
