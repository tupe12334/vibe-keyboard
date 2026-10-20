use image::{ImageBuffer, Rgb, RgbImage};

use super::super::text::draw_label;
use super::{inset_key, FALLBACK_BG, FALLBACK_FG, KEY_SIZE, NAV_BG};
use crate::domain::profile::KeyInset;

pub(in crate::infrastructure::images) fn label_tile(
    label: &str,
    bg: [u8; 3],
    fg: [u8; 3],
) -> RgbImage {
    let mut img = ImageBuffer::from_pixel(KEY_SIZE, KEY_SIZE, Rgb(bg));
    draw_label(&mut img, label, Rgb(fg));
    img
}

/// Plain label tile on the default colors. Used for dynamic lists such as Claude sessions.
pub fn label_image(label: &str, inset: &KeyInset) -> RgbImage {
    inset_key(
        &label_tile(label, FALLBACK_BG, FALLBACK_FG),
        inset,
        FALLBACK_BG,
    )
}

/// A dimmed label tile for an action that does not work here, such as a text prompt in VS Code.
pub fn muted_image(label: &str, inset: &KeyInset) -> RgbImage {
    inset_key(
        &label_tile(label, [25, 25, 25], [90, 90, 90]),
        inset,
        [25, 25, 25],
    )
}

/// A red label tile for a destructive action, such as EXIT.
pub fn danger_image(label: &str, inset: &KeyInset) -> RgbImage {
    inset_key(
        &label_tile(label, [170, 30, 30], [255, 255, 255]),
        inset,
        [170, 30, 30],
    )
}

/// Keys 11, 12 and 13 always navigate. Each shows a fixed label.
pub(super) fn nav_image(key: u8) -> RgbImage {
    let label = match key {
        11 => "BACK",
        12 => "OUT",
        _ => "FWD",
    };
    label_tile(label, NAV_BG, [200, 200, 200])
}
