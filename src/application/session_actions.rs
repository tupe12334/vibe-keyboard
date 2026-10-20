use std::sync::{Arc, Mutex};

use tracing::info;

use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::claude_sessions::{
    blink_session, focus_session, open_new_session, send_prompt,
};
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

use super::input_modes::{self, NumberEdit};
use super::key_action::Action;
use super::pages::{open_confirm_exit, open_session_actions, show_focus_only};
use super::{prompt_return, render_screen, DeviceState};

/// Claude session keys, sorting, paging and number entry.
pub(super) fn session_action(
    action: Action,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    match action {
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
        Action::OpenConfirmExit { session } => {
            info!("claude sessions: confirm exit for {}", session.tty);
            open_confirm_exit(nav, session);
            render_screen(nav, handle, state, dev_state);
        }
        Action::BlinkSession { session } => {
            info!("claude sessions: blink {}", session.tty);
            // Off the device loop: the focus and the flash both wait on osascript.
            std::thread::spawn(move || blink_session(&session.tty));
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
            input_modes::edit_number(NumberEdit::Digit(digit), nav, handle, state, dev_state);
        }
        Action::InputNumberClear => {
            input_modes::edit_number(NumberEdit::Clear, nav, handle, state, dev_state);
        }
        Action::InputNumberBackspace => {
            input_modes::edit_number(NumberEdit::Backspace, nav, handle, state, dev_state);
        }
        Action::Search => input_modes::start_filter_input(state),
        _ => {}
    }
}
