use std::sync::{Arc, Mutex};

use super::pages::RECAP_TEXT;
use super::{lock, prompt_return};
use crate::domain::actions::NewSessionTerminal;
use crate::domain::navigation::{prompt_index, NavigationStack, Screen};
use crate::presentation::tui;

pub(super) enum Action {
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

/// Decides what the key press does on the current screen. Main pages never reach this:
/// their keys come from the profile and are handled first.
pub(super) fn action_for_key(
    key: u8,
    nav: &NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
) -> Action {
    match nav.current() {
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
    }
}
