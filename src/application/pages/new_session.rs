use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::imageops::FilterType;
use image::{DynamicImage, RgbImage};

use crate::domain::actions::{ButtonAction, ScreenView};
use crate::infrastructure::app_icon::app_icon;
use crate::infrastructure::images::label_image;
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

const KEY_SIZE: u32 = 100;

/// The terminal choice for a new Claude session: key 1 iTerm2, key 2 Terminal.app, key 11 OUT (drawn by the page renderer).
/// Each app shows its icon, or its name when the icon is missing.
pub fn render_new_session_choice(handle: &Device, state: &Arc<Mutex<tui::AppState>>) {
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    actions.insert(
        1,
        ButtonAction {
            name: "iTerm".into(),
            title: "New in iTerm".into(),
            description: "Open Claude Code in a new iTerm2 window".into(),
        },
    );
    actions.insert(
        2,
        ButtonAction {
            name: "Terminal".into(),
            title: "New in Terminal".into(),
            description: "Open Claude Code in a new Terminal window".into(),
        },
    );
    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.actions = actions;
        s.screen = ScreenView::NewSessionChoice;
    }
    let tiles = [
        (1, app_tile("com.googlecode.iterm2", "iTerm")),
        (2, app_tile("com.apple.Terminal", "Terminal")),
    ];
    for (key, img) in tiles {
        send_button_image(handle, key, DynamicImage::ImageRgb8(img));
    }
}

/// The app icon scaled to a key, or a label tile when the icon is not available.
fn app_tile(bundle_id: &str, fallback: &str) -> RgbImage {
    app_icon(bundle_id, 85).map_or_else(
        || label_image(fallback),
        |icon| {
            icon.resize_to_fill(KEY_SIZE, KEY_SIZE, FilterType::Triangle)
                .to_rgb8()
        },
    )
}
