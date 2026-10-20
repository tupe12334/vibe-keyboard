use image::{imageops, ImageBuffer, Rgb, RgbImage};

use super::super::text::text_height;
use super::widgets::CAPTION_SCALE;

pub(super) const BAR_PAD: u32 = 4;
pub(super) const BAR_VALUE_SCALE: u32 = 3;
pub(super) const BAR_X: u32 = 30;
pub(super) const BAR_W: u32 = 40;
pub(super) const TRACK: [u8; 3] = [60, 60, 60];
pub(super) const GREEN: [u8; 3] = [46, 160, 67];
pub(super) const AMBER: [u8; 3] = [230, 160, 30];
pub(super) const RED: [u8; 3] = [220, 60, 50];

/// Rows `(top, bottom)` of the CPU bar in a slot of `slot_h`, bottom exclusive. The caption
/// sits above it and the percent text below it.
pub(super) fn bar_span(slot_h: u32) -> (u32, u32) {
    let top = text_height(CAPTION_SCALE) + 2 * BAR_PAD;
    let bottom = slot_h.saturating_sub(2 * BAR_PAD + text_height(BAR_VALUE_SCALE));
    (top, bottom.max(top))
}

/// Track, then the filled part in the threshold color. The fill rises from the bottom.
pub(super) fn draw_bar(slot: &mut RgbImage, percent: u8) {
    let percent = percent.min(100);
    let (top, bottom) = bar_span(slot.height());
    let height = bottom - top;
    fill_rect(slot, BAR_X, top, BAR_W, height, TRACK);
    let filled = height * u32::from(percent) / 100;
    fill_rect(
        slot,
        BAR_X,
        bottom - filled,
        BAR_W,
        filled,
        bar_color(percent),
    );
}

fn fill_rect(img: &mut RgbImage, x: u32, y: u32, width: u32, height: u32, color: [u8; 3]) {
    let rect = ImageBuffer::from_pixel(width, height, Rgb(color));
    imageops::replace(img, &rect, i64::from(x), i64::from(y));
}

/// Green below 60 %, amber below 85 %, red from 85 % up.
pub(super) const fn bar_color(percent: u8) -> [u8; 3] {
    match percent {
        0..=59 => GREEN,
        60..=84 => AMBER,
        _ => RED,
    }
}
