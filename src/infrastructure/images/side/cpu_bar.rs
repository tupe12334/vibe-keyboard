use image::{imageops, ImageBuffer, Rgb, RgbImage};

use super::super::text::text_height;
use super::widgets::CAPTION_SCALE;
use crate::domain::profile::{
    parse_color, BarStyle, BAR_THICKNESS_DEFAULT, BAR_THICKNESS_MAX, BAR_THICKNESS_MIN,
};

pub(super) const BAR_PAD: u32 = 4;
pub(super) const BAR_VALUE_SCALE: u32 = 3;
/// Horizontal center of the bar in the slot. A thicker bar grows out of it on both sides.
pub(super) const BAR_CENTER: u32 = 50;
pub(super) const BAR_W: u32 = BAR_THICKNESS_DEFAULT;
// Same values as the domain's `BAR_TRACK_DEFAULT` and `BAR_FILL_DEFAULT`. A test checks it.
pub(super) const TRACK: [u8; 3] = [60, 60, 60];
pub(super) const GREEN: [u8; 3] = [46, 160, 67];
pub(super) const AMBER: [u8; 3] = [230, 160, 30];
pub(super) const RED: [u8; 3] = [220, 60, 50];

/// The bar after its style is applied. A missing or invalid style field gives today's value.
pub(super) struct BarLook {
    /// `None` keeps the green, amber and red threshold colors.
    pub fill: Option<[u8; 3]>,
    pub track: [u8; 3],
    pub width: u32,
    /// Color of the percent text under the bar. `None` hides the text.
    pub value_color: Option<[u8; 3]>,
}

impl BarLook {
    /// `text` is the side screen's text color, the fallback for the percent text.
    pub(super) fn resolve(style: Option<&BarStyle>, text: [u8; 3]) -> Self {
        let default = BarStyle::default();
        let style = style.unwrap_or(&default);
        let color = |field: &Option<String>| field.as_deref().and_then(parse_color);
        let label = color(&style.label_color).unwrap_or(text);
        Self {
            fill: color(&style.fill),
            track: color(&style.track).unwrap_or(TRACK),
            width: style
                .thickness
                .unwrap_or(BAR_W)
                .clamp(BAR_THICKNESS_MIN, BAR_THICKNESS_MAX),
            value_color: style.show_value.unwrap_or(true).then_some(label),
        }
    }
}

/// Rows `(top, bottom)` of the CPU bar in a slot of `slot_h`, bottom exclusive. The caption
/// sits above it. The percent text sits below it, when shown.
pub(super) fn bar_span(slot_h: u32, show_value: bool) -> (u32, u32) {
    let top = text_height(CAPTION_SCALE) + 2 * BAR_PAD;
    let below = if show_value {
        2 * BAR_PAD + text_height(BAR_VALUE_SCALE)
    } else {
        2 * BAR_PAD
    };
    let bottom = slot_h.saturating_sub(below);
    (top, bottom.max(top))
}

/// Track, then the filled part. The fill rises from the bottom.
pub(super) fn draw_bar(slot: &mut RgbImage, percent: u8, look: &BarLook) {
    let percent = percent.min(100);
    let (top, bottom) = bar_span(slot.height(), look.value_color.is_some());
    let height = bottom - top;
    let x = BAR_CENTER.saturating_sub(look.width / 2);
    fill_rect(slot, x, top, look.width, height, look.track);
    let filled = height * u32::from(percent) / 100;
    let color = look.fill.unwrap_or_else(|| bar_color(percent));
    fill_rect(slot, x, bottom - filled, look.width, filled, color);
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
