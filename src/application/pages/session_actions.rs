use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ScreenView};
use crate::infrastructure::claude_sessions::SESSION_PROMPTS;
use crate::infrastructure::images::label_image;
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// The action page for one session: key 1 focus, keys 2 and 3 send the prompts, key 12 back.
/// Key 11 (out) is drawn by the page renderer.
pub fn render_session_actions(label: &str, handle: &Device, state: &Arc<Mutex<tui::AppState>>) {
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
        actions.insert(
            key,
            ButtonAction {
                name: (*name).into(),
                title: (*name).into(),
                description: format!("Send {text} to this session"),
            },
        );
        tiles.push((key, label_image(&name.to_uppercase())));
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
            label: label.to_string(),
        };
    }
    for (key, img) in tiles {
        send_button_image(handle, key, DynamicImage::ImageRgb8(img));
    }
}
