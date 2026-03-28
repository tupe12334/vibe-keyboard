mod pages;

pub use pages::{activate_page, page_actions};

use rusb::{Context, DeviceHandle};
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::domain::keys::raw_to_logical;
use crate::domain::navigation::Navigator;
use crate::infrastructure::persistence::DeviceState;
use crate::presentation::tui;
use pages::{open_config_in_vscode, open_log_file, open_terminal};

pub fn handle_key_event(
    buf: &[u8],
    handle: &DeviceHandle<Context>,
    nav: &mut Navigator,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let raw_id = buf[9];
    let state_byte = buf[10];
    if raw_id == 0x00 || raw_id == 0xFF { return; }
    let state_str = match state_byte { 1 => "pressed", 2 => "released", _ => return };
    let key = match raw_to_logical(raw_id) {
        Some(k) => k,
        None => return,
    };
    {
        let mut s = state.lock().unwrap();
        s.pressed_key = if state_byte == 1 { Some(key) } else { None };
    }
    if state_byte == 1 {
        info!("key {:2} {state_str}", key);
    }
    if state_byte != 1 { return; }
    match key {
        11 => {
            let page = nav.back();
            info!("← back → page {}", page + 1);
            activate_page(page, handle, state, dev_state);
        }
        12 => {
            let page = nav.forward();
            info!("→ forward → page {}", page + 1);
            activate_page(page, handle, state, dev_state);
        }
        _ => {
            if nav.current() == 0 && key == 2 {
                info!("opening Terminal");
                open_terminal();
            } else if nav.current() == 1 && key == 14 {
                info!("opening log file in VS Code");
                open_log_file();
            } else if nav.current() == 1 && key == 15 {
                info!("opening config in VS Code");
                open_config_in_vscode();
            }
        }
    }
}
