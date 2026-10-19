use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ClaudeSession, ScreenView};
use crate::infrastructure::images::label_image;
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// Keys 1..=10 show sessions. Keys 14 and 15 page when there are more than 10.
pub fn render_claude_sessions(
    sessions: &[ClaudeSession],
    page: usize,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    const PER_PAGE: usize = 10;
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    let mut tiles = Vec::new();

    for (i, session) in sessions
        .iter()
        .skip(page * PER_PAGE)
        .take(PER_PAGE)
        .enumerate()
    {
        let key = (i + 1) as u8;
        actions.insert(
            key,
            ButtonAction {
                name: session.label.clone(),
                title: "Claude Code".into(),
                description: format!("/dev/{}", session.tty),
            },
        );
        tiles.push((key, label_image(&session.label)));
    }
    if sessions.len() > PER_PAGE {
        actions.insert(
            14,
            ButtonAction {
                name: "Prev".into(),
                title: "Previous Sessions".into(),
                description: "Previous page of sessions".into(),
            },
        );
        actions.insert(
            15,
            ButtonAction {
                name: "Next".into(),
                title: "Next Sessions".into(),
                description: "Next page of sessions".into(),
            },
        );
        tiles.push((14, label_image("PREV")));
        tiles.push((15, label_image("NEXT")));
    }

    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.actions = actions;
        s.screen = ScreenView::ClaudeSessions {
            total: sessions.len(),
            page,
        };
    }

    for (key, img) in tiles {
        send_button_image(handle, key, DynamicImage::ImageRgb8(img));
    }
}
