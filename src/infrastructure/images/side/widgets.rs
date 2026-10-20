use image::{imageops, ImageBuffer, Rgb, RgbImage};

use super::super::text::{draw_label_scaled, draw_line, text_height};
use super::cpu_bar::{draw_bar, BAR_PAD, BAR_VALUE_SCALE};
use super::value_text::{free_text, percent, temperature_text};
use crate::domain::profile::{PixelRect, Widget};
use crate::infrastructure::system_metrics::Metrics;

/// The text each widget shows, in order. Equal text means equal pixels, so the tick can skip a
/// redraw when nothing it shows changed.
pub fn widget_texts(widgets: &[Widget], metrics: &Metrics) -> Vec<String> {
    widgets
        .iter()
        .map(|widget| match widget {
            Widget::CpuPercent | Widget::CpuBar => percent(metrics.cpu_percent),
            Widget::Temperature => temperature_text(metrics.temperature),
            Widget::DiskFree => free_text(metrics.disk),
            Widget::Text { value } => value.clone(),
        })
        .collect()
}

// Caption + value block, centered in a widget slot. Scales as `draw_line` takes them.
pub(super) const CAPTION_SCALE: u32 = 2;
const VALUE_SCALE: u32 = 5;
const BLOCK_GAP: u32 = 4;

/// Stacks the widgets top to bottom in the `stack` box, one equal slot per widget. Each
/// widget draws into its own slot, so text that is too tall is clipped there.
pub(super) fn draw_widgets(
    canvas: &mut RgbImage,
    widgets: &[Widget],
    metrics: &Metrics,
    bg: [u8; 3],
    fg: [u8; 3],
    stack: PixelRect,
) {
    // `widgets` is never empty here; `max(1)` only keeps the division safe.
    let count = widgets.len().max(1) as u32;
    let slot_h = stack.h / count;
    for (i, widget) in widgets.iter().enumerate() {
        let mut slot = ImageBuffer::from_pixel(stack.w, slot_h, Rgb(bg));
        draw_widget(&mut slot, widget, metrics, fg);
        let y = stack.y + i as u32 * slot_h;
        imageops::replace(canvas, &slot, i64::from(stack.x), i64::from(y));
    }
}

fn draw_widget(slot: &mut RgbImage, widget: &Widget, metrics: &Metrics, fg: [u8; 3]) {
    let fg = Rgb(fg);
    match widget {
        Widget::CpuPercent => caption_value(slot, "CPU", &percent(metrics.cpu_percent), fg),
        Widget::CpuBar => {
            draw_line(slot, BAR_PAD, "CPU", fg, CAPTION_SCALE);
            draw_bar(slot, metrics.cpu_percent);
            let y = slot
                .height()
                .saturating_sub(BAR_PAD + text_height(BAR_VALUE_SCALE));
            let text = percent(metrics.cpu_percent);
            draw_line(slot, y, &text, fg, BAR_VALUE_SCALE);
        }
        Widget::Temperature => {
            caption_value(slot, "TEMP", &temperature_text(metrics.temperature), fg);
        }
        Widget::DiskFree => caption_value(slot, "DISK", &free_text(metrics.disk), fg),
        Widget::Text { value } => draw_label_scaled(slot, value, fg, VALUE_SCALE),
    }
}

/// Small caption over a large value, the pair centered in the slot.
fn caption_value(slot: &mut RgbImage, caption: &str, value: &str, fg: Rgb<u8>) {
    let caption_h = text_height(CAPTION_SCALE);
    let block = caption_h + BLOCK_GAP + text_height(VALUE_SCALE);
    let top = slot.height().saturating_sub(block) / 2;
    draw_line(slot, top, caption, fg, CAPTION_SCALE);
    draw_line(slot, top + caption_h + BLOCK_GAP, value, fg, VALUE_SCALE);
}
