use image::DynamicImage;
use rusb::{Context, DeviceHandle};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::domain::actions::ButtonAction;
use crate::infrastructure::images::{generate_terminal_image, generate_vscode_config_image};
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

pub fn open_config_in_vscode() {
    let config_path = {
        let mut p = if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            std::path::PathBuf::from(xdg)
        } else {
            let mut home = std::path::PathBuf::from(
                std::env::var_os("HOME").expect("HOME not set"),
            );
            home.push(".config");
            home
        };
        p.push("vibe-keyboard");
        p.push("state.toml");
        p
    };
    Command::new("code")
        .arg(config_path)
        .spawn()
        .unwrap_or_else(|e| { eprintln!("Failed to open config in VS Code: {e}"); std::process::exit(1) });
}

pub fn open_terminal() {
    Command::new("open")
        .arg("-n")
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
            send_button_image(handle, 15, DynamicImage::ImageRgb8(generate_vscode_config_image()));
            state.lock().unwrap().push_log("page 1: vscode config".into());
        }
        _ => {}
    }
}
