use std::collections::BTreeMap;
use std::path::Path;

use image::{ImageBuffer, Rgb, RgbImage};

use super::fit;

mod app_icon_cache;
mod session_tile;
mod sources;
mod tiles;
use crate::domain::keys::is_nav_key;
use crate::domain::profile::KEY_COUNT;
use crate::domain::profile::{parse_color, Action, KeyConfig, KeyInset};
use crate::infrastructure::usb::DECK_IMAGE_SIZE;
pub use app_icon_cache::app_icon_generation;
pub use session_tile::session_image;
use sources::builtin_icon;
pub(super) use sources::source_image;
pub use tiles::danger_image;
pub(super) use tiles::label_tile;
use tiles::nav_image;
pub use tiles::{label_image, muted_image};

/// Render size before the inset step. Icons and labels are drawn at this size.
pub(super) const KEY_SIZE: u32 = 100;
pub(super) const FALLBACK_BG: [u8; 3] = [20, 20, 20];
pub(super) const FALLBACK_FG: [u8; 3] = [230, 230, 230];
pub(super) const BLANK: [u8; 3] = [0, 0, 0];
const NAV_BG: [u8; 3] = [40, 40, 40];

/// The bitmap the deck shows for `key`, `DECK_IMAGE_SIZE` square. The controller sends this
/// image, and the editor previews this same image. Rotation is not applied here: it is a
/// device transport detail.
pub fn key_image(key: u8, config: Option<&KeyConfig>, root: &Path, inset: &KeyInset) -> RgbImage {
    if is_nav_key(key) {
        return inset_key(&nav_image(key), inset, NAV_BG);
    }
    match config {
        Some(config) => {
            let bg = parse_color(&config.bg).unwrap_or(FALLBACK_BG);
            inset_key(&render_key(config, root), inset, bg)
        }
        None => ImageBuffer::from_pixel(DECK_IMAGE_SIZE, DECK_IMAGE_SIZE, Rgb(BLANK)),
    }
}

/// The one place key insets apply. `picture` is the full key render. Its own uniform border is
/// trimmed, then the content scales to the largest size that fits the visible rect, centered.
/// The hidden band keeps the key color `bg`.
pub(super) fn inset_key(picture: &RgbImage, inset: &KeyInset, bg: [u8; 3]) -> RgbImage {
    fit::fit_into_key(picture, inset, bg, true)
}

/// Every key 1–15 of one page, in key order. This is exactly what the controller sends.
pub fn page_images(
    keys: &BTreeMap<u8, KeyConfig>,
    root: &Path,
    inset: &KeyInset,
) -> Vec<(u8, RgbImage)> {
    (1..=KEY_COUNT)
        .map(|key| (key, key_image(key, keys.get(&key), root, inset)))
        .collect()
}

/// Picture for a configured key. Order: `image` if set, then the built-in icon for `Builtin`
/// actions, then the label on the key's colors. Relative images resolve against `root`.
fn render_key(config: &KeyConfig, root: &Path) -> RgbImage {
    if let Some(img) = config
        .image
        .as_deref()
        .and_then(|image| source_image(image, root, KEY_SIZE))
    {
        return img;
    }
    if let Action::Builtin { name } = &config.action {
        if let Some(icon) = builtin_icon(name) {
            return icon;
        }
    }
    let bg = parse_color(&config.bg).unwrap_or(FALLBACK_BG);
    let fg = parse_color(&config.fg).unwrap_or(FALLBACK_FG);
    label_tile(&config.label, bg, fg)
}

#[cfg(test)]
#[path = "profile_key_tests.rs"]
mod tests;
