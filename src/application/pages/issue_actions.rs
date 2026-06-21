use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;
use rusb::{Context, DeviceHandle};

use crate::domain::actions::{ButtonAction, CentyIssue, ScreenView};
use crate::infrastructure::images::{
    generate_centy_image, generate_vscode_config_image, generate_web_image,
};
use crate::infrastructure::usb::send_button_image;
use crate::presentation::tui;

pub fn render_issue_actions(
    issue: &CentyIssue,
    project_name: &str,
    org: &str,
    handle: &DeviceHandle<Context>,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    actions.insert(
        1,
        ButtonAction {
            name: "VSCode".into(),
            title: "Open in VS Code".into(),
            description: issue.file_path.as_deref().unwrap_or("no local file").into(),
        },
    );
    actions.insert(
        2,
        ButtonAction {
            name: "Web".into(),
            title: "Open in Web".into(),
            description: format!(
                "https://app.centy.io/{}/{}/issues/{}",
                org, project_name, issue.id
            ),
        },
    );
    actions.insert(
        3,
        ButtonAction {
            name: "Workspace".into(),
            title: "Open Centy Workspace".into(),
            description: format!("cockpit for {project_name}"),
        },
    );

    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.actions = actions;
        s.screen = ScreenView::CentyIssueActions {
            issue_number: issue.number,
            project_name: project_name.to_string(),
        };
    }

    send_button_image(
        handle,
        1,
        DynamicImage::ImageRgb8(generate_vscode_config_image()),
    );
    send_button_image(handle, 2, DynamicImage::ImageRgb8(generate_web_image()));
    send_button_image(handle, 3, DynamicImage::ImageRgb8(generate_centy_image()));
}
