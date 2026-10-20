use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use tracing::{error, info};

use super::builtin::run_builtin;
use crate::application::render_screen;
use crate::domain::hotkey::{parse_hotkey, push_to_talk_key};
use crate::domain::navigation::NavigationStack;
use crate::domain::profile::{is_bundle_id, Action};
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
