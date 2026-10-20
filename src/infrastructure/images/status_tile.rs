//! Key tiles with a coloured status bar on top: the Claude session tile and the Moadim routine tile.

use image::{ImageBuffer, Rgb, RgbImage};

use super::profile_key::{inset_key, BLANK, FALLBACK_BG, FALLBACK_FG, KEY_SIZE};
use super::text::{draw_tag, draw_title};
use crate::domain::moadim::RoutineStatus;
use crate::domain::profile::KeyInset;

/// A Moadim routine tile: a status bar in the status colour with its short word, then the title.
pub fn routine_image(title: &str, status: RoutineStatus, inset: &KeyInset) -> RgbImage {
    let (bar, word) = match status {
        RoutineStatus::Running => ([60, 120, 255], "RUN"),
        RoutineStatus::Enabled => ([40, 180, 90], "ON"),
        RoutineStatus::Disabled => ([110, 110, 110], "OFF"),
        RoutineStatus::Failed => ([220, 40, 40], "FAIL"),
    };
    inset_key(&status_tile(title, bar, word), inset, FALLBACK_BG)
}

/// A key with a coloured bar on top (the tag in the bar's right corner) and the title below.
pub(super) fn status_tile(title: &str, bar: [u8; 3], tag: &str) -> RgbImage {
    const BAR_H: u32 = 12;
    let mut body = ImageBuffer::from_pixel(KEY_SIZE, KEY_SIZE - BAR_H, Rgb(FALLBACK_BG));
    draw_title(&mut body, &title.to_uppercase(), Rgb(FALLBACK_FG));
    let mut img = ImageBuffer::from_pixel(KEY_SIZE, KEY_SIZE, Rgb(bar));
    draw_tag(&mut img, tag, Rgb(BLANK));
    image::imageops::replace(&mut img, &body, 0, i64::from(BAR_H));
    img
}
