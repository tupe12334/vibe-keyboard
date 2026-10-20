use image::{Rgb, RgbImage};

use super::super::draw::fill_circle;
use super::super::status_tile::status_tile;
use super::{inset_key, FALLBACK_BG, KEY_SIZE};
use crate::domain::actions::SessionStatus;
use crate::domain::profile::KeyInset;
/// A Claude session tile: a status bar on top (green active, orange stale), the age in the bar's
/// right corner, and the title below, wrapped to three lines.
/// `focused` adds a white frame and a dot in the corner. The bar keeps its status colour.
pub fn session_image(
    title: &str,
    status: SessionStatus,
    age: &str,
    inset: &KeyInset,
    focused: bool,
) -> RgbImage {
    let bar = match status {
        SessionStatus::Active => [40, 180, 90],
        SessionStatus::Stale => [255, 120, 0],
    };
    let mut tile = status_tile(title, bar, age);
    if focused {
        draw_focus_marker(&mut tile);
    }
    inset_key(&tile, inset, FALLBACK_BG)
}

/// White frame of 3 px around the tile, and a white dot in the bottom-left corner.
fn draw_focus_marker(img: &mut RgbImage) {
    const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
    for (x, y, px) in img.enumerate_pixels_mut() {
        if x < 3 || y < 3 || x >= KEY_SIZE - 3 || y >= KEY_SIZE - 3 {
            *px = WHITE;
        }
    }
    fill_circle(img, 12, 87, 6, WHITE);
}
