use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ClaudeSession, ScreenView, TerminalApp};
use crate::domain::navigation::{
    action_pages, prompt_index, NavigationStack, Screen, PROMPTS_PER_PAGE,
};

use crate::infrastructure::images::{label_image, muted_image};
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// Text that key 14 sends to a session. Fixed, so it stays off the prompt list.
pub const RECAP_TEXT: &str = "/recap";

/// The action page for one session. Keys 1 to 10 send the prompts for this page. Key 11 (out) is
/// drawn by the page renderer, and key 12 goes back to the list. Key 13 stays empty, reserved for
/// voice. Key 14 sends Recap. Key 15 turns to the next prompt page, when there is one. Prompts and
/// Recap are muted for apps that take no text.
pub fn render_session_actions(
    session: &ClaudeSession,
    page: usize,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let prompts = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .profile
        .session_prompts();
    let typed = session.app != TerminalApp::Other;
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    let mut tiles = vec![(12, label_image("BACK"))];
    for key in 1..=(PROMPTS_PER_PAGE as u8) {
        let Some(index) = prompt_index(page, key, prompts.len()) else {
            continue;
        };
        let prompt = &prompts[index];
        let description = if typed {
            format!("Send {} to this session", prompt.text)
        } else {
            "Focus only: this app takes no typed text".into()
        };
        actions.insert(
            key,
            ButtonAction {
                name: prompt.label.clone(),
                title: prompt.label.clone(),
                description,
            },
        );
        let label = prompt.label.clone();
        tiles.push((
            key,
            if typed {
                label_image(&label)
            } else {
                muted_image(&label)
            },
        ));
    }
    tiles.push((
        14,
        if typed {
            label_image("RECAP")
        } else {
            muted_image("RECAP")
        },
    ));
    actions.insert(
        14,
        ButtonAction {
            name: "Recap".into(),
            title: "Recap".into(),
            description: if typed {
                format!("Send {RECAP_TEXT} to this session")
            } else {
                "Recap: this app takes no typed text".into()
            },
        },
    );
    if action_pages(prompts.len()) > 1 {
        tiles.push((15, label_image("MORE")));
    }
    actions.insert(
        12,
        ButtonAction {
            name: "Back".into(),
            title: "Back".into(),
            description: "Back to the sessions list".into(),
        },
    );
    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.actions = actions;
        s.screen = ScreenView::SessionActions {
            label: session.label.clone(),
        };
    }
    for (key, img) in tiles {
        send_button_image(handle, key, DynamicImage::ImageRgb8(img));
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::SessionStatus;

    #[test]
    fn selecting_a_session_focuses_its_tab_then_opens_actions() {
        let session = ClaudeSession {
            pid: 1,
            app: TerminalApp::ITerm,
            tty: "ttys009".into(),
            label: "s".into(),
            status: SessionStatus::Active,
            age: None,
        };
        let mut nav = NavigationStack::new(0, 1);
        let mut focused = Vec::new();
        open_session_actions(&mut nav, session, |tty| focused.push(tty.to_string()));
        assert_eq!(focused, ["ttys009"]);
        assert!(matches!(
            nav.current(),
            Screen::SessionActions { page: 0, .. }
        ));
    }
}
