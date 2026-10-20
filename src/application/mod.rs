mod actions;
mod moadim_keys;
mod pages;
pub mod prompt_return;

pub use pages::{refresh_side_widgets, render_screen};
pub use refresh::Refresh;

mod refresh;

mod dictation;
mod frontmost;
mod gestures;
mod holds;

pub use frontmost::handle_frontmost;
use gestures::end_press;
use holds::track_press;
pub use holds::{install_panic_release, poll_holds, read_wait, release_all_holds};

use crate::domain::actions::{NewSessionTerminal, TerminalApp};
use crate::domain::navigation::prompt_index;
use crate::infrastructure::claude_sessions::{focus_session, open_new_session, send_prompt};
use crate::infrastructure::usb::Device;
use actions::{open_centy_workspace, open_in_chrome, open_terminal_in_path, open_vscode_in_path};
use pages::{fetch_centy_issues, open_session_actions, show_focus_only, RECAP_TEXT};
use std::sync::{Arc, Mutex, MutexGuard};
use tracing::info;

use crate::domain::keys::raw_to_logical;
use crate::domain::navigation::{back_key_on, BackKey, NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::presentation::tui;

fn lock(state: &Arc<Mutex<tui::AppState>>) -> MutexGuard<'_, tui::AppState> {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn handle_key_event(
    buf: &[u8],
    handle: &Device,
    nav: &mut NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
    refresh: &mut Refresh,
) {
    let raw_id = buf[9];
    let state_byte = buf[10];
    if raw_id == 0x00 || raw_id == 0xFF {
        return;
    }
    let state_str = match state_byte {
        1 => "pressed",
        0 => "released",
        _ => return,
    };
    let Some(key) = raw_to_logical(raw_id) else {
        return;
    };
    // Release always clears the highlight, even while loading.
    if state_byte != 1 {
        lock(state).pressed_key = None;
        end_press(key, handle, nav, state, dev_state);
        return;
    }
    if lock(state).loading {
        return;
    }
    lock(state).pressed_key = Some(key);
    info!("key {:2} {state_str}", key);
    track_press(key, nav, state);

    let is_input_number = matches!(nav.current(), Screen::InputNumber { .. });
    if matches!(key, 11 | 13) && nav.main_page().is_some() {
        // Paging by hand on a main page makes it manual, so app switches stop fighting it.
        lock(state).auto_page.on_manual_page();
    }

    match key {
        11 => {
            match back_key_on(nav.current()) {
                BackKey::Back => nav.back(),
                BackKey::Out => nav.out_to_main(),
            }
            render_screen(nav, handle, state, dev_state);
        }
        12 => {
            // On an action page, key 12 goes back to the session list. Elsewhere it goes out.
            if matches!(nav.current(), Screen::SessionActions { .. }) {
                prompt_return::back_to_sessions(nav);
            } else {
                nav.out();
            }
            render_screen(nav, handle, state, dev_state);
        }
        // On the InputNumber screen key 13 maps to digit 7, not "forward".
        _ if nav.is_moadim() => {
            moadim_keys::handle_key(key, nav, handle, state, dev_state, &mut refresh.moadim);
        }
        13 if !is_input_number && !matches!(nav.current(), Screen::ClaudeSessions { .. }) => {
            nav.forward();
            render_screen(nav, handle, state, dev_state);
        }
        _ => handle_action_key(key, nav, handle, state, dev_state),
    }
}

fn handle_action_key(
    key: u8,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    // Extract what we need from the current screen before any mutable nav operations.
    enum Action {
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
            org: String,
        },
        SelectIssue {
            issues: Vec<crate::domain::actions::CentyIssue>,
            project_name: String,
            org: String,
            idx: usize,
        },
        OpenIssueInVsCode {
            file_path: Option<String>,
        },
        OpenIssueInWeb {
            id: String,
            org: String,
            project_name: String,
        },
        OpenCentyWorkspace {
            issue_number: u64,
        },
        SortList,
        OpenNewSessionChoice,
        OpenSessionActions {
            session: crate::domain::actions::ClaudeSession,
        },
        SendPrompt {
            pid: u32,
            tty: String,
            text: String,
        },
        PromptUnsupported {
            key: u8,
        },
        TurnActionPage {
            forward: bool,
            count: usize,
        },

        NewSession {
            terminal: NewSessionTerminal,
        },
        TurnSessionsPage {
            forward: bool,
        },
        InputNumberDigit {
            digit: char,
        },
        InputNumberClear,
        InputNumberBackspace,
        Search,
        None,
    }

    // Main page keys come from the user's profile, not from code.
    if let Screen::MainPage { page } = nav.current() {
        let page = *page;
        // A key with `hold` waits for its gesture. `end_press` runs the tap.
        let configured = lock(state)
            .profile
            .key_at(page, key)
            .filter(|config| config.hold.is_none())
            .map(|config| config.action.clone());
        if let Some(action) = configured {
            actions::run_action(&action, nav, handle, state, dev_state);
        }
        return;
    }

    let action = match nav.current() {
        // Moadim pages are handled by `moadim_keys` before this match.
        Screen::MainPage { .. } | Screen::MoadimList { .. } | Screen::MoadimRoutine { .. } => {
            Action::None
        }
        Screen::CentyProjectList { projects, page, .. } => {
            if key == 15 {
                Action::Search
            } else if matches!(key, 1..=10) {
                let idx = page * 10 + (key as usize - 1);
                if projects.get(idx).is_some() {
                    Action::SelectProject {
                        projects: projects.clone(),
                        idx,
                    }
                } else {
                    Action::None
                }
            } else if key == 14 {
                Action::SortList
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
                org: project.org.clone(),
            },
            _ => Action::None,
        },
        Screen::CentyIssueList {
            issues,
            page,
            project_name,
            org,
            ..
        } => {
            if key == 15 {
                Action::Search
            } else if matches!(key, 1..=10) {
                let idx = page * 10 + (key as usize - 1);
                if issues.get(idx).is_some() {
                    Action::SelectIssue {
                        issues: issues.clone(),
                        project_name: project_name.clone(),
                        org: org.clone(),
                        idx,
                    }
                } else {
                    Action::None
                }
            } else if key == 14 {
                Action::SortList
            } else {
                Action::None
            }
        }
        Screen::CentyIssueActions {
            issue,
            project_name,
            org,
        } => match key {
            1 => Action::OpenIssueInVsCode {
                file_path: issue.file_path.clone(),
            },
            2 => Action::OpenIssueInWeb {
                id: issue.id.clone(),
                org: org.clone(),
                project_name: project_name.clone(),
            },
            3 => Action::OpenCentyWorkspace {
                issue_number: issue.number,
            },
            _ => Action::None,
        },
        Screen::ClaudeSessions { sessions, page } => match key {
            1..=10 => {
                let idx = page * 10 + (key as usize - 1);
                sessions
                    .get(idx)
                    .map_or(Action::None, |session| Action::OpenSessionActions {
                        session: session.clone(),
                    })
            }
            13 => Action::OpenNewSessionChoice,
            14 => Action::TurnSessionsPage { forward: false },
            15 => Action::TurnSessionsPage { forward: true },
            _ => Action::None,
        },
        Screen::SessionActions { session, page } => {
            let prompts = lock(state).profile.session_prompts();
            let text = match key {
                1..=10 => {
                    prompt_index(*page, key, prompts.len()).map(|index| prompts[index].text.clone())
                }
                14 => Some(RECAP_TEXT.into()),
                _ => None,
            };
            match (key, text) {
                (15, _) => Action::TurnActionPage {
                    forward: true,
                    count: prompts.len(),
                },
                (_, None) => Action::None,
                (_, Some(_)) if !prompt_return::may_send(session.app) => {
                    Action::PromptUnsupported { key }
                }
                (_, Some(text)) => Action::SendPrompt {
                    pid: session.pid,
                    tty: session.tty.clone(),
                    text,
                },
            }
        }
        Screen::NewSessionChoice => match key {
            1 => Action::NewSession {
                terminal: NewSessionTerminal::ITerm,
            },
            2 => Action::NewSession {
                terminal: NewSessionTerminal::Terminal,
            },
            _ => Action::None,
        },
        Screen::InputNumber { .. } => match key {
            1 => Action::InputNumberClear,
            2 => Action::InputNumberBackspace,
            3 => Action::InputNumberDigit { digit: '1' },
            4 => Action::InputNumberDigit { digit: '2' },
            5 => Action::InputNumberDigit { digit: '3' },
            8 => Action::InputNumberDigit { digit: '4' },
            9 => Action::InputNumberDigit { digit: '5' },
            10 => Action::InputNumberDigit { digit: '6' },
            13 => Action::InputNumberDigit { digit: '7' },
            14 => Action::InputNumberDigit { digit: '8' },
            15 => Action::InputNumberDigit { digit: '9' },
            _ => Action::None,
        },
    };

    match action {
        Action::SelectProject { projects, idx } => {
            if let Some(project) = projects.into_iter().nth(idx) {
                info!("centy: selected project {}", project.name);
                nav.push(Screen::CentyProjectActions { project });
                render_screen(nav, handle, state, dev_state);
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
        Action::FetchIssues { project_name, org } => {
            lock(state).loading = true;
            let issues = fetch_centy_issues(&project_name);
            lock(state).loading = false;
            if !issues.is_empty() {
                nav.push(Screen::CentyIssueList {
                    issues,
                    page: 0,
                    project_name,
                    org,
                    filter: None,
                });
                render_screen(nav, handle, state, dev_state);
            }
        }
        Action::SelectIssue {
            issues,
            project_name,
            org,
            idx,
        } => {
            if let Some(issue) = issues.into_iter().nth(idx) {
                info!("centy: selected issue #{}", issue.number);
                nav.push(Screen::CentyIssueActions {
                    issue,
                    project_name,
                    org,
                });
                render_screen(nav, handle, state, dev_state);
            }
        }
        Action::OpenIssueInVsCode { file_path } => {
            let path = file_path.as_deref().unwrap_or(".");
            info!("centy: open issue in VS Code at {}", path);
            open_vscode_in_path(path);
        }
        Action::OpenIssueInWeb {
            id,
            org,
            project_name,
        } => {
            let url = format!("https://app.centy.io/{org}/{project_name}/issues/{id}");
            info!("centy: open issue in web: {}", url);
            open_in_chrome(&url);
        }
        Action::OpenCentyWorkspace { issue_number } => {
            info!("centy: opening workspace for issue {}", issue_number);
            open_centy_workspace(issue_number);
        }
        Action::SortList => {
            info!("sort: toggling sort order");
            nav.toggle_sort();
            render_screen(nav, handle, state, dev_state);
        }
        Action::OpenSessionActions { session } => {
            info!("claude sessions: actions for {}", session.tty);
            // Selecting a session focuses its tab first, then opens the action page.
            open_session_actions(nav, session, |tty| {
                let tty = tty.to_string();
                // Off the device loop: osascript can take a while to answer.
                std::thread::spawn(move || focus_session(&tty));
            });
            render_screen(nav, handle, state, dev_state);
        }
        Action::SendPrompt { pid, tty, text } => {
            info!("claude sessions: send prompt to {tty}");
            // Off the device loop: osascript can take a while. Leaves the page once it is sent.
            prompt_return::start_prompt(tty.clone(), move || send_prompt(pid, &tty, &text));
        }
        Action::TurnActionPage { forward, count } => {
            nav.turn_action_page(forward, count);
            render_screen(nav, handle, state, dev_state);
        }
        Action::PromptUnsupported { key } => {
            info!("claude sessions: key {key} needs iTerm2 or Terminal; focus only");
            show_focus_only(handle, key, state);
        }
        Action::OpenNewSessionChoice => {
            info!("claude sessions: choose a terminal for a new session");
            nav.push(Screen::NewSessionChoice);
            render_screen(nav, handle, state, dev_state);
        }
        Action::NewSession { terminal } => {
            info!("claude sessions: new session in {terminal:?}");
            // Off the device loop: osascript can take a while to answer.
            std::thread::spawn(move || open_new_session(terminal));
            // Back to the list. The new session shows on the next refresh.
            nav.back();
            render_screen(nav, handle, state, dev_state);
        }
        Action::TurnSessionsPage { forward } => {
            if forward {
                nav.forward();
            } else {
                nav.previous_page();
            }
            render_screen(nav, handle, state, dev_state);
        }
        Action::InputNumberDigit { digit } => {
            nav.input_number_append(digit);
            let value = nav.input_number_value().unwrap_or("").to_string();
            info!("input number: appended '{}', value = {}", digit, value);
            render_screen(nav, handle, state, dev_state);
        }
        Action::InputNumberClear => {
            nav.input_number_clear();
            info!("input number: cleared");
            render_screen(nav, handle, state, dev_state);
        }
        Action::InputNumberBackspace => {
            nav.input_number_backspace();
            let value = nav.input_number_value().unwrap_or("").to_string();
            info!("input number: backspace, value = {}", value);
            render_screen(nav, handle, state, dev_state);
        }
        Action::Search => {
            info!("activating filter input mode");
            let mut s = lock(state);
            s.text_input_mode = true;
            s.text_input_value = String::new();
            s.text_input_confirmed = false;
        }
        Action::None => {}
    }
}

pub fn handle_filter_query(
    query: &str,
    handle: &Device,
    nav: &mut NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    use crate::domain::actions::{CentyIssue, CentyProject};

    enum FilterAction {
        Projects {
            projects: Vec<CentyProject>,
            filter: Option<String>,
        },
        Issues {
            issues: Vec<CentyIssue>,
            project_name: String,
            org: String,
            filter: Option<String>,
        },
        None,
    }

    let q = query.to_lowercase();
    let filter = if q.trim().is_empty() {
        None
    } else {
        Some(query.to_string())
    };

    let fa = match nav.current() {
        Screen::CentyProjectList { projects, .. } => {
            let filtered: Vec<CentyProject> = if q.trim().is_empty() {
                projects.clone()
            } else {
                projects
                    .iter()
                    .filter(|p| {
                        p.name.to_lowercase().contains(&q) || p.org.to_lowercase().contains(&q)
                    })
                    .cloned()
                    .collect()
            };
            FilterAction::Projects {
                projects: filtered,
                filter,
            }
        }
        Screen::CentyIssueList {
            issues,
            project_name,
            org,
            ..
        } => {
            let filtered: Vec<CentyIssue> = if q.trim().is_empty() {
                issues.clone()
            } else {
                issues
                    .iter()
                    .filter(|i| {
                        i.title.to_lowercase().contains(&q)
                            || i.number.to_string().contains(&q)
                            || i.status.to_lowercase().contains(&q)
                    })
                    .cloned()
                    .collect()
            };
            FilterAction::Issues {
                issues: filtered,
                project_name: project_name.clone(),
                org: org.clone(),
                filter,
            }
        }
        _ => FilterAction::None,
    };

    match fa {
        FilterAction::Projects { projects, filter } => {
            info!(
                "filter applied: {} project(s) match {:?}",
                projects.len(),
                query
            );
            nav.push(Screen::CentyProjectList {
                projects,
                page: 0,
                filter,
            });
            render_screen(nav, handle, state, dev_state);
        }
        FilterAction::Issues {
            issues,
            project_name,
            org,
            filter,
        } => {
            info!(
                "filter applied: {} issue(s) match {:?}",
                issues.len(),
                query
            );
            nav.push(Screen::CentyIssueList {
                issues,
                page: 0,
                project_name,
                org,
                filter,
            });
            render_screen(nav, handle, state, dev_state);
        }
        FilterAction::None => {}
    }
}
