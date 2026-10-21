use image::RgbImage;

use crate::application::voice::VOICE_LABEL;
use crate::domain::actions::{ButtonAction, ClaudeSession};
use crate::domain::navigation::{exit_on_page, NavigationStack, Screen};
use crate::domain::profile::KeyInset;
use crate::infrastructure::images::{label_image, muted_image};

use super::RECAP_TEXT;
use crate::application::pages::confirm_exit::exit_key;

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
        label_image(VOICE_LABEL, inset)
    } else {
        muted_image(VOICE_LABEL, inset)
    };
    let action = ButtonAction {
        name: "Voice".into(),
        title: "Voice".into(),
        description: if typed {
            "Hold: dictate. Tap: send (Enter)".into()
        } else {
            "Voice: this app takes no typed text".into()
        },
    };
    (tile, action)
}

/// Key 15: MORE on every page but the last, EXIT on the last page. Muted for apps that take no text.
pub fn more_or_exit_key(
    page: usize,
    count: usize,
    typed: bool,
    inset: &KeyInset,
) -> (RgbImage, ButtonAction) {
    if exit_on_page(page, count) {
        return exit_key(typed, inset);
    }
    let action = ButtonAction {
        name: "More".into(),
        title: "More".into(),
        description: "Next page of prompts".into(),
    };
    (label_image("MORE", inset), action)
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
