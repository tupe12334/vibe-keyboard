use image::{DynamicImage, RgbImage};

use crate::domain::actions::{ButtonAction, ClaudeSession};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::images::{label_image, muted_image};
use crate::infrastructure::usb::{send_button_image, Device};

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

/// Shows a short message on a prompt key that the session's app cannot take. It stays until the
/// user leaves the page.
pub fn show_focus_only(handle: &Device, key: u8) {
    send_button_image(
        handle,
        key,
        DynamicImage::ImageRgb8(label_image("FOCUS ONLY")),
    );
}

/// The Recap key (14): its tile and its action. Muted for apps that take no typed text.
pub fn recap_key(typed: bool) -> (RgbImage, ButtonAction) {
    let tile = if typed {
        label_image("RECAP")
    } else {
        muted_image("RECAP")
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
