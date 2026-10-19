use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ClaudeSession, ScreenView, TerminalApp};
use crate::infrastructure::claude_sessions::SESSION_PROMPTS;
use crate::infrastructure::images::{label_image, muted_image};
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// The action page for one session: key 1 focus, keys 2 and 3 send the prompts, key 12 back.
/// Key 11 (out) is drawn by the page renderer. Prompts are muted for apps that take no text.
pub fn render_session_actions(
    session: &ClaudeSession,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let typed = session.app != TerminalApp::Other;
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    let mut tiles = vec![(1, label_image("FOCUS")), (12, label_image("BACK"))];
    actions.insert(
        1,
        ButtonAction {
            name: "Focus".into(),
            title: "Focus".into(),
            description: "Show this session's tab".into(),
        },
    );
    for (i, (name, text)) in SESSION_PROMPTS.iter().enumerate() {
        let key = (i + 2) as u8;
        let description = if typed {
            format!("Send {text} to this session")
        } else {
            "Focus only: this app takes no typed text".into()
        };
        actions.insert(
            key,
            ButtonAction {
                name: (*name).into(),
                title: (*name).into(),
                description,
            },
        );
        let label = name.to_uppercase();
        tiles.push((
            key,
            if typed {
                label_image(&label)
            } else {
                muted_image(&label)
            },
        ));
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

/// Shows a short message on a prompt key that the session's app cannot take. It stays until the
/// user leaves the page.
pub fn show_focus_only(handle: &Device, key: u8) {
    send_button_image(
        handle,
        key,
        DynamicImage::ImageRgb8(label_image("FOCUS ONLY")),
    );
}
