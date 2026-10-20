//! The bar widgets: one percent bar, and one bar split between CPU and memory.

use image::Rgb;
use image::RgbImage;

use super::super::text::{draw_line, draw_line_within, text_height};
use super::cpu_bar::{
    bar_span, draw_bar, draw_column, BarLook, BAR_CENTER, BAR_PAD, BAR_VALUE_SCALE,
};
use super::value_text::percent;
use super::widgets::CAPTION_SCALE;

/// Caption, bar and percent. `caption` and the percent take the bar's look.
pub(super) fn draw_percent_bar(
    slot: &mut RgbImage,
    caption: &str,
    value: u8,
    look: &BarLook,
    fg: Rgb<u8>,
) {
    draw_line(slot, BAR_PAD, caption, fg, CAPTION_SCALE);
    draw_bar(slot, value, look);
    if let Some(color) = look.value_color {
        let y = slot
            .height()
            .saturating_sub(BAR_PAD + text_height(BAR_VALUE_SCALE));
        draw_line(slot, y, &percent(value), Rgb(color), BAR_VALUE_SCALE);
    }
}

/// Width of the CPU half and the memory half of a bar `width` px wide. The left half takes an odd pixel.
pub(super) const fn halves(width: u32) -> (u32, u32) {
    let left = width.div_ceil(2);
    (left, width - left)
}

const SPLIT_LABEL_SCALE: u32 = 1;
const SPLIT_VALUE_SCALE: u32 = 2;

/// One bar `look.width` px wide, split: CPU on the left, memory on the right. Each half fills
/// to its own percent. With the value label on, each half gets its name above and its number below.
pub(super) fn draw_split(slot: &mut RgbImage, cpu: u8, memory: u8, look: &BarLook) {
    let (top, bottom) = bar_span(slot.height(), look.value_color.is_some());
    let x0 = BAR_CENTER.saturating_sub(look.width / 2);
    let (left_w, right_w) = halves(look.width);
    let value_y = slot
        .height()
        .saturating_sub(BAR_PAD + text_height(SPLIT_VALUE_SCALE));
    let parts = [
        (x0, left_w, cpu, "CPU"),
        (x0 + left_w, right_w, memory, "MEM"),
    ];
    for (x, width, value, name) in parts {
        draw_column(slot, x, width, top, bottom, value, look);
        if let Some(color) = look.value_color {
            let color = Rgb(color);
            draw_line_within(slot, x, width, BAR_PAD, name, color, SPLIT_LABEL_SCALE);
            draw_line_within(
                slot,
                x,
                width,
                value_y,
                &value.to_string(),
                color,
                SPLIT_VALUE_SCALE,
            );
        }
    }
}
