use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ClaudeSession, ScreenView, TerminalApp};
use crate::domain::navigation::{action_pages, prompt_index, PROMPTS_PER_PAGE};
use crate::domain::profile::KeyInset;
use crate::infrastructure::images::{label_image, muted_image};
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// The action page for one session. Key 1 focuses the tab. Keys 2 to 10 send the prompts for
/// this page. Keys 14 and 15 page through prompts. Key 11 (out) is drawn by the page renderer,
/// and key 12 goes back to the list. Prompts are muted for apps that take no text.
pub fn render_session_actions(
    session: &ClaudeSession,
    page: usize,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let inset = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .profile
        .display
        .key_inset;
    let prompts = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .profile
        .session_prompts();
    let typed = session.app != TerminalApp::Other;
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    let mut tiles = vec![
        (1, label_image("FOCUS", &inset)),
        (12, label_image("BACK", &inset)),
    ];
    actions.insert(
        1,
        ButtonAction {
            name: "Focus".into(),
            title: "Focus".into(),
            description: "Show this session's tab".into(),
        },
    );
    for key in 2..=(PROMPTS_PER_PAGE as u8 + 1) {
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
                label_image(&label, &inset)
            } else {
                muted_image(&label, &inset)
            },
        ));
    }
    let pages = action_pages(prompts.len());
    if page > 0 {
        tiles.push((14, label_image("PREV", &inset)));
    }
    if page + 1 < pages {
        tiles.push((15, label_image("NEXT", &inset)));
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
pub fn show_focus_only(handle: &Device, key: u8, inset: &KeyInset) {
    send_button_image(
        handle,
        key,
        DynamicImage::ImageRgb8(label_image("FOCUS ONLY", inset)),
    );
}
