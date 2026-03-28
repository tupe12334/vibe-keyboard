use image::DynamicImage;
use rusb::{Context, DeviceHandle};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::domain::actions::{ButtonAction, CentyProject, CentyState};
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

pub fn run_centy_projects(state: &Arc<Mutex<tui::AppState>>, handle: &DeviceHandle<Context>) {
    info!("running centy list projects");

    let output = Command::new("pnpm")
        .args(["dlx", "centy", "list", "projects", "--json"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            info!("centy error: {e}");
            return;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        info!("centy failed: {}", stderr.trim());
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let projects = parse_centy_json(&stdout);

    if projects.is_empty() {
        info!("centy: no projects found");
        return;
    }

    info!("centy: {} project(s)", projects.len());
    show_project_list(0, projects, state, handle);
}

fn parse_centy_json(json: &str) -> Vec<CentyProject> {
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    // Accept either a top-level array or an object with a "projects" key
    let arr: Vec<&serde_json::Value> = if let Some(a) = value.as_array() {
        a.iter().collect()
    } else if let Some(a) = value.get("projects").and_then(|v| v.as_array()) {
        a.iter().collect()
    } else {
        return vec![];
    };

    arr.into_iter()
        .filter_map(|item| {
            let name = ["name", "slug", "title"]
                .iter()
                .find_map(|k| item.get(k)?.as_str())
                .unwrap_or("unknown")
                .to_string();

            let org = [
                "organizationSlug",
                "organizationName",
                "org",
                "organization",
                "owner",
                "workspace",
            ]
            .iter()
            .find_map(|k| item.get(k)?.as_str())
            .unwrap_or("unknown")
            .to_string();

            let path = ["path", "directory", "dir", "root", "localPath"]
                .iter()
                .find_map(|k| item.get(k)?.as_str())
                .map(|s| s.to_string());

            let url = format!("https://app.centy.io/{}/{}", org, name);

            Some(CentyProject {
                name,
                org,
                path,
                url,
            })
        })
        .collect()
}

pub fn show_project_list(
    page: usize,
    projects: Vec<CentyProject>,
    state: &Arc<Mutex<tui::AppState>>,
    handle: &DeviceHandle<Context>,
) {
    const PER_PAGE: usize = 10;
    let start = page * PER_PAGE;
    let page_slice: Vec<&CentyProject> = projects.iter().skip(start).take(PER_PAGE).collect();

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
        s.centy_state = Some(CentyState::ProjectList { projects, page });
    }

    clear_all(handle);
    let project_img = DynamicImage::ImageRgb8(generate_project_item_image());
    for i in 0..count {
        send_button_image(handle, (i + 1) as u8, project_img.clone());
    }
}

pub fn show_project_actions(
    project: CentyProject,
    state: &Arc<Mutex<tui::AppState>>,
    handle: &DeviceHandle<Context>,
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

    {
        let mut s = state.lock().unwrap();
        s.actions = actions;
        s.centy_state = Some(CentyState::ProjectActions { project });
    }

    clear_all(handle);
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
        s.centy_state = None;
    }
    {
        let mut ds = dev_state.lock().unwrap();
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
