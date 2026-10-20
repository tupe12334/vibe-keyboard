//! Draws the exit confirmation screen. The behavior lives in `confirm_exit.rs`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use super::confirm_exit::EXIT_TEXT;
use super::key_tiles;
use crate::application::lock;
use crate::domain::actions::{ButtonAction, ClaudeSession, ScreenView};
use crate::infrastructure::images::{danger_image, label_image};
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// Draws the confirm screen: the question on key 1, CONFIRM EXIT on 13, CANCEL on 11 and 12.
pub fn render_confirm_exit(
    session: &ClaudeSession,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let inset = key_tiles::key_inset(state);
    let cancel = || ButtonAction {
        name: "Cancel".into(),
        title: "Cancel".into(),
        description: "Back to the session. Nothing is sent".into(),
    };
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    actions.insert(
        13,
        ButtonAction {
            name: "Confirm exit".into(),
            title: "Confirm exit".into(),
            description: format!("Send {EXIT_TEXT} to this session"),
        },
    );
    actions.insert(11, cancel());
    actions.insert(12, cancel());
    {
        let mut s = lock(state);
        s.actions = actions;
        s.screen = ScreenView::ConfirmExit {
            label: session.label.clone(),
        };
    }
    let tiles = [
        (1, label_image(&format!("Exit {}?", session.label), &inset)),
        (11, label_image("CANCEL", &inset)),
        (12, label_image("CANCEL", &inset)),
        (13, danger_image("CONFIRM EXIT", &inset)),
    ];
    for (key, img) in tiles {
        send_button_image(handle, key, &DynamicImage::ImageRgb8(img));
    }
}
