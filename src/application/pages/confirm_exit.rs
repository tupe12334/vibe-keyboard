//! The exit confirmation. EXIT (key 15, last action page) opens it. CONFIRM EXIT (13) sends `/exit`,
//! and CANCEL (11 or 12) goes back to the session page without sending.

use image::RgbImage;

use crate::application::key_action::Action;
use crate::application::prompt_return;
use crate::domain::actions::{ButtonAction, ClaudeSession};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::domain::profile::KeyInset;
use crate::infrastructure::images::{danger_image, muted_image};

/// Text that CONFIRM EXIT sends to the session.
pub const EXIT_TEXT: &str = "/exit";

/// EXIT pressed on a session page: show the confirmation above it.
pub fn open_confirm_exit(nav: &mut NavigationStack, session: ClaudeSession) {
    nav.push(Screen::ConfirmExit { session });
}

/// Key 15 on the last action page: the EXIT tile and its action. Muted for apps that take no text.
pub fn exit_key(typed: bool, inset: &KeyInset) -> (RgbImage, ButtonAction) {
    let tile = if typed {
        danger_image("EXIT", inset)
    } else {
        muted_image("EXIT", inset)
    };
    let action = ButtonAction {
        name: "Exit".into(),
        title: "Exit".into(),
        description: if typed {
            "Exit this session. Asks to confirm first".into()
        } else {
            "Exit: this app takes no typed text".into()
        },
    };
    (tile, action)
}

/// EXIT pressed on a session page. Muted apps do nothing.
pub fn exit_action(session: &ClaudeSession) -> Action {
    if prompt_return::may_send(session.app) {
        Action::OpenConfirmExit {
            session: session.clone(),
        }
    } else {
        Action::None
    }
}

/// Keys on the confirm screen. Only CONFIRM EXIT (13) sends. Cancel (11, 12) never reaches here.
pub fn confirm_action(key: u8, session: &ClaudeSession) -> Action {
    match key {
        13 if prompt_return::may_send(session.app) => Action::SendPrompt {
            pid: session.pid,
            tty: session.tty.clone(),
            text: EXIT_TEXT.into(),
        },
        _ => Action::None,
    }
}

#[cfg(test)]
#[path = "confirm_exit_tests.rs"]
mod tests;
