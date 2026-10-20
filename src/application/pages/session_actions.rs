use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ClaudeSession, ScreenView, TerminalApp};
use crate::domain::navigation::{action_pages, prompt_index, PROMPTS_PER_PAGE};

use crate::application::voice::VOICE_KEY;
use crate::infrastructure::images::{label_image, muted_image};
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// Text that key 14 sends to a session. Fixed, so it stays off the prompt list.
pub const RECAP_TEXT: &str = "/recap";

/// The action page for one session. Keys 1 to 10 send the prompts for this page. Key 11 (out) is
/// drawn by the page renderer, and key 12 goes back to the list. Key 13 is voice: hold to talk.
/// Key 14 sends Recap. Key 15 turns to the next prompt page, when there is one. Prompts, Recap and
/// voice are muted for apps that take no text.
pub fn render_session_actions(
    session: &ClaudeSession,
    page: usize,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let inset = super::key_tiles::key_inset(state);
    let prompts = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .profile
        .session_prompts();
    let typed = session.app != TerminalApp::Other;
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    let mut tiles = vec![(12, label_image("BACK", &inset))];
    let (voice_tile, voice) = voice_key(typed, &inset);
    tiles.push((VOICE_KEY, voice_tile));
    actions.insert(VOICE_KEY, voice);
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
                label_image(&label, &inset)
            } else {
                muted_image(&label, &inset)
            },
        ));
    }
    let (recap_tile, recap) = recap_key(typed, &inset);
    tiles.push((14, recap_tile));
    actions.insert(14, recap);
    if action_pages(prompts.len()) > 1 {
        tiles.push((15, label_image("MORE", &inset)));
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

#[path = "session_actions_keys.rs"]
mod keys;
pub use keys::open_session_actions;
use keys::{recap_key, voice_key};

#[cfg(test)]
#[path = "session_actions_tests.rs"]
mod tests;
