use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tracing::{error, info, warn};

use super::builtin::run_builtin;
use super::effects;
use crate::application::render_screen;
use crate::domain::accessibility::hotkey_log;
use crate::domain::hotkey::{parse_hotkey, push_to_talk_key};
use crate::domain::navigation::NavigationStack;
use crate::domain::profile::{is_bundle_id, Action};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// Set after the first key combo logs that it lacks the grant, so the line prints once.
static HOTKEY_LOGGED: AtomicBool = AtomicBool::new(false);

/// Run the action bound to a main page key.
pub fn run_action(
    action: &Action,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let fx = effects::current();
    match action {
        Action::None => {}
        Action::OpenUrl { url } => {
            info!("profile: open url {url}");
            (fx.spawn)("open", &["--", url.as_str()]);
        }
        Action::RunCommand { command } => {
            info!("profile: run command");
            (fx.spawn)("sh", &["-c", command.as_str()]);
        }
        Action::OpenApp { name } => {
            info!("profile: open app {name}");
            if is_bundle_id(name) {
                (fx.spawn)("open", &["-b", name.as_str()]);
            } else {
                (fx.spawn)("open", &["-a", name.as_str()]);
            }
        }
        Action::Hotkey { keys } => match parse_hotkey(keys) {
            Ok(hotkey) => {
                info!("profile: hotkey {keys}");
                if let Some(line) = hotkey_log((fx.trusted)()) {
                    if !HOTKEY_LOGGED.swap(true, Ordering::SeqCst) {
                        warn!("{line}");
                    }
                }
                let script = hotkey.applescript();
                (fx.spawn)("osascript", &["-e", script.as_str()]);
            }
            Err(e) => error!("profile: bad hotkey '{keys}': {e}"),
        },
        Action::PushToTalk { key } => match push_to_talk_key(key.as_deref()) {
            Ok(hotkey) => {
                info!("profile: push-to-talk tap");
                (fx.tap)(&hotkey);
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
