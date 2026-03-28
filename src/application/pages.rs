use image::DynamicImage;
use rusb::{Context, DeviceHandle};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::domain::actions::ButtonAction;
use crate::infrastructure::images::generate_terminal_image;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::{clear_all, send_button_image};
use crate::presentation::tui;

pub fn page_actions(page: usize) -> HashMap<u8, ButtonAction> {
    let mut map = HashMap::new();
    match page {
        0 => {
            map.insert(2, ButtonAction {
                name: "Terminal",
                title: "Open Terminal",
                description: "Launch a new terminal window",
            });
        }
        _ => {}
    }
    map
}

pub fn open_terminal() {
    Command::new("open")
        .arg("-a")
        .arg("Terminal")
        .spawn()
        .unwrap_or_else(|e| { eprintln!("Failed to open Terminal: {e}"); std::process::exit(1) });
}

pub fn activate_page(
    page: usize,
    handle: &DeviceHandle<Context>,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    {
        let mut s = state.lock().unwrap();
        s.current_page = page;
        s.actions = page_actions(page);
    }
    {
        let mut ds = dev_state.lock().unwrap();
        ds.current_page = page;
        ds.save();
    }
    clear_all(handle);
    match page {
        0 => {
            send_button_image(handle, 2, DynamicImage::ImageRgb8(generate_terminal_image()));
            state.lock().unwrap().push_log("page 0: terminal".into());
        }
        1 => {
            state.lock().unwrap().push_log("page 1: (empty)".into());
        }
        _ => {}
    }
}
