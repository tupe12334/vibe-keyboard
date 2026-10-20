use std::sync::{Arc, Mutex};

use super::Action;
use crate::application::pages::{confirm_action, exit_action, RECAP_TEXT};
use crate::application::{lock, prompt_return};
use crate::domain::actions::NewSessionTerminal;
use crate::domain::navigation::{exit_on_page, prompt_index, Screen, BLINK_KEY};
use crate::presentation::tui;

/// Keys on the Claude session screens: the list, the action page and the new-session choice.
pub(super) fn action(key: u8, screen: &Screen, state: &Arc<Mutex<tui::AppState>>) -> Action {
    match screen {
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
                (BLINK_KEY, _) => Action::BlinkSession {
                    session: session.clone(),
                },
                (15, _) if exit_on_page(*page, prompts.len()) => exit_action(session),
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
        Screen::ConfirmExit { session } => confirm_action(key, session),
        Screen::NewSessionChoice => match key {
            1 => Action::NewSession {
                terminal: NewSessionTerminal::ITerm,
            },
            2 => Action::NewSession {
                terminal: NewSessionTerminal::Terminal,
            },
            _ => Action::None,
        },
        _ => Action::None,
    }
}
