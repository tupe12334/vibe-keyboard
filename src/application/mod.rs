mod actions;
mod pages;

pub use pages::render_screen;

use actions::{
    open_claude_terminal, open_config_in_vscode, open_in_chrome, open_log_file, open_terminal,
    open_terminal_in_path, open_vscode_in_path,
};
use pages::{fetch_centy_issues, fetch_centy_projects};
use rusb::{Context, DeviceHandle};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::domain::keys::raw_to_logical;
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::presentation::tui;

pub fn handle_key_event(
    buf: &[u8],
    handle: &DeviceHandle<Context>,
    nav: &mut NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let raw_id = buf[9];
    let state_byte = buf[10];
    if raw_id == 0x00 || raw_id == 0xFF {
        return;
    }
    let state_str = match state_byte {
        1 => "pressed",
        2 => "released",
        _ => return,
    };
    let key = match raw_to_logical(raw_id) {
        Some(k) => k,
        None => return,
    };
    if state.lock().unwrap_or_else(|e| e.into_inner()).loading {
        return;
    }
    {
        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
        s.pressed_key = if state_byte == 1 { Some(key) } else { None };
    }
    if state_byte == 1 {
        info!("key {:2} {state_str}", key);
    }
    if state_byte != 1 {
        return;
    }

    match key {
        11 => {
            nav.back();
            render_screen(nav.current(), handle, state, dev_state);
        }
        12 => {
            nav.out();
            render_screen(nav.current(), handle, state, dev_state);
        }
        13 => {
            nav.forward();
            render_screen(nav.current(), handle, state, dev_state);
        }
        _ => handle_action_key(key, nav, handle, state, dev_state),
    }
}

fn handle_action_key(
    key: u8,
    nav: &mut NavigationStack,
    handle: &DeviceHandle<Context>,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    // Extract what we need from the current screen before any mutable nav operations.
    enum Action {
        FetchProjects,
        OpenTerminal,
        OpenClaude,
        OpenCentyWeb,
        OpenLogFile,
        OpenConfig,
        SelectProject {
            projects: Vec<crate::domain::actions::CentyProject>,
            idx: usize,
        },
        OpenVsCode {
            name: String,
            path: Option<String>,
        },
        OpenTerminalInPath {
            name: String,
            path: Option<String>,
        },
        OpenBrowser {
            name: String,
            url: String,
        },
        FetchIssues {
            project_name: String,
        },
        SelectIssue {
            issues: Vec<crate::domain::actions::CentyIssue>,
            project_name: String,
            idx: usize,
        },
        OpenIssueInVsCode {
            file_path: Option<String>,
        },
        OpenIssueInWeb {
            id: String,
        },
        OpenCentyCockpit,
        None,
    }

    let action = match nav.current() {
        Screen::MainPage { page } => match (*page, key) {
            (0, 1) => Action::FetchProjects,
            (0, 2) => Action::OpenTerminal,
            (0, 3) => Action::OpenClaude,
            (0, 4) => Action::OpenCentyWeb,
            (1, 14) => Action::OpenLogFile,
            (1, 15) => Action::OpenConfig,
            _ => Action::None,
        },
        Screen::CentyProjectList { projects, page } => {
            if matches!(key, 1..=10) {
                let idx = page * 10 + (key as usize - 1);
                if projects.get(idx).is_some() {
                    Action::SelectProject {
                        projects: projects.clone(),
                        idx,
                    }
                } else {
                    Action::None
                }
            } else {
                Action::None
            }
        }
        Screen::CentyProjectActions { project } => match key {
            1 => Action::OpenVsCode {
                name: project.name.clone(),
                path: project.path.clone(),
            },
            2 => Action::OpenTerminalInPath {
                name: project.name.clone(),
                path: project.path.clone(),
            },
            3 => Action::OpenBrowser {
                name: project.name.clone(),
                url: project.url.clone(),
            },
            4 => Action::FetchIssues {
                project_name: project.name.clone(),
            },
            _ => Action::None,
        },
        Screen::CentyIssueList {
            issues,
            page,
            project_name,
        } => {
            if matches!(key, 1..=10) {
                let idx = page * 10 + (key as usize - 1);
                if issues.get(idx).is_some() {
                    Action::SelectIssue {
                        issues: issues.clone(),
                        project_name: project_name.clone(),
                        idx,
                    }
                } else {
                    Action::None
                }
            } else {
                Action::None
            }
        }
        Screen::CentyIssueActions { issue, .. } => match key {
            1 => Action::OpenIssueInVsCode {
                file_path: issue.file_path.clone(),
            },
            2 => Action::OpenIssueInWeb {
                id: issue.id.clone(),
            },
            3 => Action::OpenCentyCockpit,
            _ => Action::None,
        },
    };

    match action {
        Action::FetchProjects => {
            state.lock().unwrap_or_else(|e| e.into_inner()).loading = true;
            let projects = fetch_centy_projects();
            state.lock().unwrap_or_else(|e| e.into_inner()).loading = false;
            if !projects.is_empty() {
                nav.push(Screen::CentyProjectList { projects, page: 0 });
                render_screen(nav.current(), handle, state, dev_state);
            }
        }
        Action::OpenTerminal => {
            info!("opening Terminal");
            open_terminal();
        }
        Action::OpenClaude => {
            info!("opening Claude in Terminal");
            open_claude_terminal();
        }
        Action::OpenCentyWeb => {
            info!("opening Centy in Chrome");
            open_in_chrome("https://app.centy.io");
        }
        Action::OpenLogFile => {
            info!("opening log file in VS Code");
            open_log_file();
        }
        Action::OpenConfig => {
            info!("opening config in VS Code");
            open_config_in_vscode();
        }
        Action::SelectProject { projects, idx } => {
            if let Some(project) = projects.into_iter().nth(idx) {
                info!("centy: selected project {}", project.name);
                nav.push(Screen::CentyProjectActions { project });
                render_screen(nav.current(), handle, state, dev_state);
            }
        }
        Action::OpenVsCode { name, path } => {
            info!("centy: open {} in VS Code", name);
            open_vscode_in_path(path.as_deref().unwrap_or("."));
        }
        Action::OpenTerminalInPath { name, path } => {
            info!("centy: open {} in Terminal", name);
            open_terminal_in_path(path.as_deref());
        }
        Action::OpenBrowser { name, url } => {
            info!("centy: open {} in browser", name);
            open_in_chrome(&url);
        }
        Action::FetchIssues { project_name } => {
            state.lock().unwrap_or_else(|e| e.into_inner()).loading = true;
            let issues = fetch_centy_issues(&project_name);
            state.lock().unwrap_or_else(|e| e.into_inner()).loading = false;
            if !issues.is_empty() {
                nav.push(Screen::CentyIssueList {
                    issues,
                    page: 0,
                    project_name,
                });
                render_screen(nav.current(), handle, state, dev_state);
            }
        }
        Action::SelectIssue {
            issues,
            project_name,
            idx,
        } => {
            if let Some(issue) = issues.into_iter().nth(idx) {
                info!("centy: selected issue #{}", issue.number);
                nav.push(Screen::CentyIssueActions {
                    issue,
                    project_name,
                });
                render_screen(nav.current(), handle, state, dev_state);
            }
        }
        Action::OpenIssueInVsCode { file_path } => {
            let path = file_path.as_deref().unwrap_or(".");
            info!("centy: open issue in VS Code at {}", path);
            open_vscode_in_path(path);
        }
        Action::OpenIssueInWeb { id } => {
            let url = format!("https://app.centy.io/{}", id);
            info!("centy: open issue in web: {}", url);
            open_in_chrome(&url);
        }
        Action::OpenCentyCockpit => {
            info!("centy: opening cockpit workspace");
            #[allow(clippy::zombie_processes)]
            let _ = Command::new("pnpm")
                .args(["dlx", "centy", "cockpit"])
                .spawn();
        }
        Action::None => {}
    }
}
