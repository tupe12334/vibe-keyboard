//! The voice key (13) on a session's action page. A hold pushes to talk (`dictation.rs`). A tap
//! only flashes HOLD on the tile. Never sends `/voice`.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use image::DynamicImage;

use super::pages::key_tiles::key_inset;
use crate::domain::actions::TerminalApp;
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::images::label_image;
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// The key on a session action page that holds voice.
pub const VOICE_KEY: u8 = 13;

/// How long a tap shows HOLD on the tile before VOICE comes back.
const FLASH: Duration = Duration::from_millis(800);

/// Whether a press of `key` on this screen is the voice hold. Apps that take no text are muted.
pub fn holds_voice(screen: &Screen, key: u8) -> bool {
    matches!(screen, Screen::SessionActions { session, .. }
        if key == VOICE_KEY && session.app != TerminalApp::Other)
}

/// A tap on the voice key: show HOLD on its tile.
pub fn flash(handle: &Device, state: &Arc<Mutex<tui::AppState>>) {
    arm(&mut super::lock(state).voice_flash, Instant::now());
    draw(handle, state, "HOLD");
}

/// Bring VOICE back once a flash has run out. Draws only on the session page, so it never
/// paints over another page's key.
pub fn expire_flash(
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    nav: &NavigationStack,
    now: Instant,
) {
    let expired = take_expired(&mut super::lock(state).voice_flash, now);
    if expired && matches!(nav.current(), Screen::SessionActions { .. }) {
        draw(handle, state, "VOICE");
    }
}

fn draw(handle: &Device, state: &Arc<Mutex<tui::AppState>>, label: &str) {
    let tile = label_image(label, &key_inset(state));
    send_button_image(handle, VOICE_KEY, &DynamicImage::ImageRgb8(tile));
}

/// Start a flash that ends `FLASH` after `now`.
fn arm(flash: &mut Option<Instant>, now: Instant) {
    *flash = Some(now + FLASH);
}

/// Clear the flash when its deadline has passed. Returns true when it did.
fn take_expired(flash: &mut Option<Instant>, now: Instant) -> bool {
    match *flash {
        Some(deadline) if now >= deadline => {
            *flash = None;
            true
        }
        _ => false,
    }
}

#[cfg(test)]
#[path = "voice_tests.rs"]
mod tests;
