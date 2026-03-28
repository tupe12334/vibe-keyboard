mod centy;

pub use centy::{run_centy_projects, show_project_actions, show_project_list};

use image::DynamicImage;
use rusb::{Context, DeviceHandle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::domain::actions::ButtonAction;
use crate::infrastructure::images::{
    generate_centy_image, generate_claude_image, generate_log_file_image, generate_terminal_image,
    generate_vscode_config_image, generate_web_image,
};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::{clear_all, send_button_image};
use crate::presentation::tui;

pub fn page_actions(page: usize) -> HashMap<u8, ButtonAction> {
    let mut map = HashMap::new();
    match page {
        0 => {
            map.insert(
                1,
                ButtonAction {
                    name: "Centy".into(),
                    title: "List Projects".into(),
                    description: "Show centy projects".into(),
                },
            );
            map.insert(
                2,
                ButtonAction {
                    name: "Terminal".into(),
                    title: "Open Terminal".into(),
                    description: "Launch a new terminal window".into(),
                },
            );
            map.insert(
                3,
                ButtonAction {
                    name: "Claude".into(),
                    title: "Open Claude".into(),
                    description: "Launch terminal with claude --allow-dangerously-skip-permissions"
                        .into(),
                },
            );
            map.insert(
                4,
                ButtonAction {
                    name: "Centy Web".into(),
                    title: "Open Centy".into(),
                    description: "Open app.centy.io in Chrome".into(),
                },
            );
        }
        1 => {
            map.insert(
                14,
                ButtonAction {
                    name: "Log File".into(),
                    title: "Open Log".into(),
                    description: "Open app.log in VS Code".into(),
                },
            );
            map.insert(
                15,
                ButtonAction {
                    name: "VSCode Config".into(),
                    title: "Open Config".into(),
                    description: "Open config file in VS Code".into(),
                },
            );
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
        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
        s.current_page = page;
        s.actions = page_actions(page);
        s.centy_state = None;
    }
    {
        let mut ds = dev_state.lock().unwrap_or_else(|e| e.into_inner());
        ds.current_page = page;
        ds.save();
    }
    clear_all(handle);
    match page {
        0 => {
            send_button_image(handle, 1, DynamicImage::ImageRgb8(generate_centy_image()));
            send_button_image(
                handle,
                2,
                DynamicImage::ImageRgb8(generate_terminal_image()),
            );
            send_button_image(handle, 3, DynamicImage::ImageRgb8(generate_claude_image()));
            send_button_image(handle, 4, DynamicImage::ImageRgb8(generate_web_image()));
            info!("page 0: centy + terminal + claude + centy web");
        }
        1 => {
            send_button_image(
                handle,
                14,
                DynamicImage::ImageRgb8(generate_log_file_image()),
            );
            send_button_image(
                handle,
                15,
                DynamicImage::ImageRgb8(generate_vscode_config_image()),
            );
            info!("page 1: log file + vscode config");
        }
        _ => {}
    }
}
