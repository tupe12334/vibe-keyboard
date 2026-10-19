use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ClaudeSession, ScreenView, SessionStatus};
use crate::infrastructure::images::{label_image, session_image};
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

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
                title: status_text(session.status).into(),
                description: format!("/dev/{}", session.tty),
            },
        );
        tiles.push((key, session_image(&session.label, session.status)));
    }
    if page > 0 {
        actions.insert(
            14,
            ButtonAction {
                name: "Back".into(),
                title: "Previous Sessions".into(),
                description: "Previous page of sessions".into(),
            },
        );
        tiles.push((14, label_image("BACK")));
    }
    if pages > 1 {
        actions.insert(
            15,
            ButtonAction {
                name: "More".into(),
                title: "Next Sessions".into(),
                description: "Next page of sessions".into(),
            },
        );
        let word = if page + 1 < pages { "MORE" } else { "PAGE" };
        tiles.push((15, label_image(&format!("{word} {}/{pages}", page + 1))));
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

/// Short status for the button description, shown in the editor and the TUI.
const fn status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Waiting => "Waiting for you",
        SessionStatus::Working => "Working",
        SessionStatus::Unknown => "Claude Code",
    }
}
