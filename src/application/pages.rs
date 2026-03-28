use image::DynamicImage;
use rusb::{Context, DeviceHandle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::domain::actions::ButtonAction;
use crate::infrastructure::images::{generate_log_file_image, generate_terminal_image, generate_vscode_config_image};
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
        1 => {
            map.insert(14, ButtonAction {
                name: "Log File",
                title: "Open Log",
                description: "Open app.log in VS Code",
            });
            map.insert(15, ButtonAction {
                name: "VSCode Config",
                title: "Open Config",
                description: "Open config file in VS Code",
            });
        }
        _ => {}
    }
    map
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
            info!("page 0: terminal");
        }
        1 => {
            send_button_image(handle, 14, DynamicImage::ImageRgb8(generate_log_file_image()));
            send_button_image(handle, 15, DynamicImage::ImageRgb8(generate_vscode_config_image()));
            info!("page 1: log file + vscode config");
        }
        _ => {}
    }
}
