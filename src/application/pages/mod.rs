mod centy;

pub use centy::{fetch_centy_issues, fetch_centy_projects};

use image::DynamicImage;
use rusb::{Context, DeviceHandle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::domain::actions::{ButtonAction, ScreenView};
use crate::domain::navigation::Screen;
use crate::infrastructure::images::{
    generate_centy_image, generate_claude_image, generate_log_file_image,
    generate_project_item_image, generate_terminal_image, generate_vscode_config_image,
    generate_web_image,
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

/// Render the given screen to both the TUI state and the hardware device.
pub fn render_screen(
    screen: &Screen,
    handle: &DeviceHandle<Context>,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    clear_all(handle);
    match screen {
        Screen::MainPage { page } => {
            {
                let mut s = state.lock().unwrap();
                s.actions = page_actions(*page);
                s.screen = ScreenView::MainPage { page: *page };
            }
            {
                let mut ds = dev_state.lock().unwrap();
                ds.current_page = *page;
                ds.save();
            }
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
        Screen::CentyProjectList { projects, page } => {
            const PER_PAGE: usize = 10;
            let start = page * PER_PAGE;
            let page_slice: Vec<&crate::domain::actions::CentyProject> =
                projects.iter().skip(start).take(PER_PAGE).collect();

            let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
            for (i, project) in page_slice.iter().enumerate() {
                let key = (i + 1) as u8;
                actions.insert(
                    key,
                    ButtonAction {
                        name: project.name.clone(),
                        title: project.org.clone(),
                        description: project.url.clone(),
                    },
                );
            }

            let count = page_slice.len();
            {
                let mut s = state.lock().unwrap();
                s.actions = actions;
                s.screen = ScreenView::CentyProjectList {
                    total: projects.len(),
                    page: *page,
                };
            }

            let project_img = DynamicImage::ImageRgb8(generate_project_item_image());
            for i in 0..count {
                send_button_image(handle, (i + 1) as u8, project_img.clone());
            }
        }
        Screen::CentyProjectActions { project } => {
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
                let mut s = state.lock().unwrap();
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
        Screen::CentyIssueList {
            issues,
            page,
            project_name,
        } => {
            const PER_PAGE: usize = 10;
            let start = page * PER_PAGE;
            let page_slice: Vec<&crate::domain::actions::CentyIssue> =
                issues.iter().skip(start).take(PER_PAGE).collect();

            let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
            for (i, issue) in page_slice.iter().enumerate() {
                let key = (i + 1) as u8;
                actions.insert(
                    key,
                    ButtonAction {
                        name: format!("#{}", issue.number),
                        title: issue.title.clone(),
                        description: issue.status.clone(),
                    },
                );
            }

            let count = page_slice.len();
            {
                let mut s = state.lock().unwrap();
                s.actions = actions;
                s.screen = ScreenView::CentyIssueList {
                    total: issues.len(),
                    page: *page,
                    project_name: project_name.clone(),
                };
            }

            let issue_img = DynamicImage::ImageRgb8(generate_project_item_image());
            for i in 0..count {
                send_button_image(handle, (i + 1) as u8, issue_img.clone());
            }
        }
    }
}
