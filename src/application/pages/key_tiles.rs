//! Key tiles drawn outside the page renderers. Each applies the profile's key inset.

use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::domain::profile::KeyInset;
use crate::infrastructure::images::label_image;
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

pub fn key_inset(state: &Arc<Mutex<tui::AppState>>) -> KeyInset {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .profile
        .display
        .key_inset
}

/// Key 11 shows OUT on every sub-screen. Drawn last, so a page cannot cover it.
pub fn show_out(handle: &Device, state: &Arc<Mutex<tui::AppState>>) {
    let inset = key_inset(state);
    let tile = label_image("OUT", &inset);
    send_button_image(handle, 11, DynamicImage::ImageRgb8(tile));
}

/// A session that cannot take prompts shows FOCUS ONLY on its key.
pub fn show_focus_only(handle: &Device, key: u8, state: &Arc<Mutex<tui::AppState>>) {
    let inset = key_inset(state);
    let tile = label_image("FOCUS ONLY", &inset);
    send_button_image(handle, key, DynamicImage::ImageRgb8(tile));
}
