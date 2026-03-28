use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;
use rusb::{Context, DeviceHandle};

use crate::domain::actions::{ButtonAction, CentyProject, ScreenView};
use crate::infrastructure::images::{
    generate_project_item_image, generate_terminal_image, generate_vscode_config_image,
    generate_web_image,
};
use crate::infrastructure::usb::send_button_image;
use crate::presentation::tui;

pub fn render_project_actions(
    project: &CentyProject,
    handle: &DeviceHandle<Context>,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    actions.insert(
        1,
        ButtonAction {
            name: "VSCode".into(),
            title: "Open in VS Code".into(),
            description: project.path.as_deref().unwrap_or("no local path").into(),
        },
    );
    actions.insert(
        2,
        ButtonAction {
            name: "Terminal".into(),
            title: "Open Terminal".into(),
            description: project.path.as_deref().unwrap_or("no local path").into(),
        },
    );
    actions.insert(
        3,
        ButtonAction {
            name: "Web".into(),
            title: "Open in Browser".into(),
            description: project.url.clone(),
        },
    );
    actions.insert(
        4,
        ButtonAction {
            name: "Issues".into(),
            title: "List Issues".into(),
            description: format!("Show issues for {}", project.name),
        },
    );

    {
        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
        s.actions = actions;
        s.screen = ScreenView::CentyProjectActions {
            project_name: project.name.clone(),
        };
    }

    send_button_image(
        handle,
        1,
        DynamicImage::ImageRgb8(generate_vscode_config_image()),
    );
    send_button_image(
        handle,
        2,
        DynamicImage::ImageRgb8(generate_terminal_image()),
    );
    send_button_image(handle, 3, DynamicImage::ImageRgb8(generate_web_image()));
    send_button_image(
        handle,
        4,
        DynamicImage::ImageRgb8(generate_project_item_image()),
    );
}
