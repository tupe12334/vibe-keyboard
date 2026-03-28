mod actions;
mod pages;

pub use pages::{activate_page, page_actions};

use actions::{
    open_claude_terminal, open_config_in_vscode, open_in_chrome, open_log_file, open_terminal,
    open_terminal_in_path, open_vscode_in_path,
};
use pages::{run_centy_projects, show_project_actions, show_project_list};
use rusb::{Context, DeviceHandle};
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::domain::actions::CentyState;
use crate::domain::keys::raw_to_logical;
use crate::domain::navigation::Navigator;
use crate::infrastructure::persistence::DeviceState;
use crate::presentation::tui;

pub fn handle_key_event(
    buf: &[u8],
    handle: &DeviceHandle<Context>,
    nav: &mut Navigator,
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
    {
        let mut s = state.lock().unwrap();
        s.pressed_key = if state_byte == 1 { Some(key) } else { None };
    }
    if state_byte == 1 {
        info!("key {:2} {state_str}", key);
    }
    if state_byte != 1 {
        return;
    }

    // Intercept key events when in centy overlay mode
    let centy_state = state.lock().unwrap().centy_state.take();
    if let Some(cs) = centy_state {
        match cs {
            CentyState::ProjectList { projects, page } => {
                match key {
                    11 => {
                        if page > 0 {
                            info!("centy: prev page {}", page);
                            show_project_list(page - 1, projects, state, handle);
                        } else {
                            info!("centy: exit → page {}", nav.current() + 1);
                            activate_page(nav.current(), handle, state, dev_state);
                        }
                    }
                    12 => {
                        info!("centy: exit → page {}", nav.current() + 1);
                        activate_page(nav.current(), handle, state, dev_state);
                    }
                    13 => {
                        let next_page = page + 1;
                        let max_page = projects.len().saturating_sub(1) / 10;
                        if next_page <= max_page {
                            info!("centy: next page {}", next_page + 1);
                            show_project_list(next_page, projects, state, handle);
                        } else {
                            // No more pages, restore state
                            state.lock().unwrap().centy_state =
                                Some(CentyState::ProjectList { projects, page });
                        }
                    }
                    1..=10 => {
                        let idx = page * 10 + (key as usize - 1);
                        if let Some(project) = projects.get(idx).cloned() {
                            info!("centy: selected project {}", project.name);
                            show_project_actions(project, projects, page, state, handle);
                        } else {
                            state.lock().unwrap().centy_state =
                                Some(CentyState::ProjectList { projects, page });
                        }
                    }
                    _ => {
                        state.lock().unwrap().centy_state =
                            Some(CentyState::ProjectList { projects, page });
                    }
                }
            }
            CentyState::ProjectActions {
                project,
                prev_projects,
                prev_page,
            } => match key {
                11 => {
                    info!("centy: back to project list (page {})", prev_page + 1);
                    show_project_list(prev_page, prev_projects, state, handle);
                }
                1 => {
                    info!("centy: open {} in VS Code", project.name);
                    open_vscode_in_path(project.path.as_deref().unwrap_or("."));
                    state.lock().unwrap().centy_state = Some(CentyState::ProjectActions {
                        project,
                        prev_projects,
                        prev_page,
                    });
                }
                2 => {
                    info!("centy: open {} in Terminal", project.name);
                    open_terminal_in_path(project.path.as_deref());
                    state.lock().unwrap().centy_state = Some(CentyState::ProjectActions {
                        project,
                        prev_projects,
                        prev_page,
                    });
                }
                3 => {
                    info!("centy: open {} in browser", project.name);
                    open_in_chrome(&project.url);
                    state.lock().unwrap().centy_state = Some(CentyState::ProjectActions {
                        project,
                        prev_projects,
                        prev_page,
                    });
                }
                _ => {
                    state.lock().unwrap().centy_state = Some(CentyState::ProjectActions {
                        project,
                        prev_projects,
                        prev_page,
                    });
                }
            },
        }
        return;
    }

    // Normal page navigation and actions
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
            if nav.current() == 0 && key == 1 {
                state.lock().unwrap().loading = true;
                run_centy_projects(state, handle);
                state.lock().unwrap().loading = false;
            } else if nav.current() == 0 && key == 2 {
                info!("opening Terminal");
                open_terminal();
            } else if nav.current() == 0 && key == 3 {
                info!("opening Claude in Terminal");
                open_claude_terminal();
            } else if nav.current() == 0 && key == 4 {
                info!("opening Centy in Chrome");
                open_in_chrome("https://app.centy.io");
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
