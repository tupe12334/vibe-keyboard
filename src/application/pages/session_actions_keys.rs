use image::RgbImage;

use crate::domain::actions::{ButtonAction, ClaudeSession};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::domain::profile::KeyInset;
use crate::infrastructure::images::{label_image, muted_image};

use super::RECAP_TEXT;

/// Selecting a session: `focus` runs first, on the session's tty, then the action page opens. The
/// device loop passes a `focus` that runs in a thread, because osascript can be slow.
pub fn open_session_actions(
    nav: &mut NavigationStack,
    session: ClaudeSession,
    focus: impl FnOnce(&str),
) {
    focus(&session.tty);
    nav.push(Screen::SessionActions { session, page: 0 });
}

/// The Voice key (13): its tile and its action. Muted for apps that take no typed text.
pub fn voice_key(typed: bool, inset: &KeyInset) -> (RgbImage, ButtonAction) {
    let tile = if typed {
        label_image("VOICE", inset)
    } else {
        muted_image("VOICE", inset)
    };
    let action = ButtonAction {
        name: "Voice".into(),
        title: "Voice".into(),
        description: if typed {
            "Hold to talk to this session. A tap only shows HOLD".into()
        } else {
            "Voice: this app takes no typed text".into()
        },
    };
    (tile, action)
}

/// The Recap key (14): its tile and its action. Muted for apps that take no typed text.
pub fn recap_key(typed: bool, inset: &KeyInset) -> (RgbImage, ButtonAction) {
    let tile = if typed {
        label_image("RECAP", inset)
    } else {
        muted_image("RECAP", inset)
    };
    let action = ButtonAction {
        name: "Recap".into(),
        title: "Recap".into(),
        description: if typed {
            format!("Send {RECAP_TEXT} to this session")
        } else {
            "Recap: this app takes no typed text".into()
        },
    };
    (tile, action)
}
