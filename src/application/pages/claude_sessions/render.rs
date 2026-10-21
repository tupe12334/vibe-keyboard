use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ClaudeSession, ScreenView};
use crate::infrastructure::claude_sessions::age_label;
use crate::infrastructure::images::session_image;
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

use super::nav_keys::add_nav_keys;
use super::status_text;

/// Keys 1..=10 show sessions, one tile each. Key 14 is BACK after the first page. Key 15 shows
/// `MORE` or `PAGE` with the page counter when there is more than one page.
pub fn render_claude_sessions(
    sessions: &[ClaudeSession],
    page: usize,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    const PER_PAGE: usize = 10;
    let pages = sessions.len().div_ceil(PER_PAGE).max(1);
    let inset = super::super::key_tiles::key_inset(state);
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    let mut tiles = Vec::new();
    for (i, entry) in sessions
        .iter()
        .skip(page * PER_PAGE)
        .take(PER_PAGE)
        .enumerate()
    {
        let key = (i + 1) as u8;
        actions.insert(
            key,
            ButtonAction {
                name: entry.label.clone(),
                title: status_text(entry.status).into(),
                description: format!("/dev/{}", entry.tty),
            },
        );
        let age = age_label(entry.age);
        let tile = session_image(
            &entry.label,
            entry.status,
            &age,
            entry.remote_control,
            &inset,
            entry.focused,
        );
        tiles.push((key, tile));
    }
    add_nav_keys(page, pages, &inset, &mut actions, &mut tiles);

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
        send_button_image(handle, key, &DynamicImage::ImageRgb8(img));
    }
}
