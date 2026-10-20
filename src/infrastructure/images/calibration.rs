//! Calibration tiles: the key number inside the visible rect, with that rect outlined.

use image::{Rgb, RgbImage};

use super::profile_key::{inset_key, label_tile};
use crate::domain::profile::{inner_rect, KeyInset};
use crate::infrastructure::usb::DECK_IMAGE_SIZE;

pub(super) const CALIBRATION_LINE: [u8; 3] = [255, 0, 0];

/// Calibration tile: the logical key number in big white digits on black, inside the visible
/// rect. A red outline marks that rect, so the user can tune the inset.
pub fn calibration_image(key: u8, inset: &KeyInset) -> RgbImage {
    let picture = label_tile(&key.to_string(), [0, 0, 0], [255, 255, 255]);
    let mut img = inset_key(&picture, inset, [0, 0, 0]);
    outline_visible(&mut img, inset);
    img
}

/// Draws a 1 px outline around the visible rect of `inset` on a deck-size image.
fn outline_visible(img: &mut RgbImage, inset: &KeyInset) {
    let rect = inner_rect(
        DECK_IMAGE_SIZE,
        DECK_IMAGE_SIZE,
        inset.top,
        inset.bottom,
        inset.left,
        inset.right,
    );
    if rect.w == 0 || rect.h == 0 {
        return;
    }
    let (x0, y0) = (rect.x, rect.y);
    let (x1, y1) = (rect.x + rect.w - 1, rect.y + rect.h - 1);
    let color = Rgb(CALIBRATION_LINE);
    for x in x0..=x1 {
        img.put_pixel(x, y0, color);
        img.put_pixel(x, y1, color);
    }
    for y in y0..=y1 {
        img.put_pixel(x0, y, color);
        img.put_pixel(x1, y, color);
    }
}
