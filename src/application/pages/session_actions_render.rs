//! Draws the action page for one session: prompt keys, Back, voice, Recap and MORE.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::application::pages::key_tiles;
use crate::application::voice::VOICE_KEY;
use crate::domain::actions::{ButtonAction, ClaudeSession, ScreenView, TerminalApp};
use crate::domain::navigation::{prompt_index, PROMPTS_PER_PAGE};
use crate::infrastructure::images::{label_image, muted_image};
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

use super::keys::{more_or_exit_key, recap_key, voice_key};

/// The action page for one session. Keys 1 to 10 send the prompts for this page. Key 11 (out) is
/// drawn by the page renderer, and key 12 goes back to the list. Key 13 is voice: hold to talk.
/// Key 14 sends Recap. Key 15 turns to the next prompt page, or on the last page opens EXIT. Prompts,
/// Recap, voice and EXIT are muted for apps that take no text.
pub fn render_session_actions(
    session: &ClaudeSession,
    page: usize,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let inset = key_tiles::key_inset(state);
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
    let (fifteen_tile, fifteen) = more_or_exit_key(page, prompts.len(), typed, &inset);
    tiles.push((15, fifteen_tile));
    actions.insert(15, fifteen);
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
        send_button_image(handle, key, &DynamicImage::ImageRgb8(img));
    }
}
