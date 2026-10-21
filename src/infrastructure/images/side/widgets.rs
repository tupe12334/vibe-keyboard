use image::{imageops, ImageBuffer, Rgb, RgbImage};

use super::value_text::{free_text, percent, temperature_text};
use super::widget_slot::draw_widget;
use crate::domain::profile::{BarStyle, PixelRect, Widget};
use crate::infrastructure::system_metrics::memory_percent;
use crate::infrastructure::system_metrics::Metrics;

/// The text each widget shows, in order. Equal text means equal pixels, so the tick can skip a
/// redraw when nothing it shows changed.
pub fn widget_texts(widgets: &[Widget], metrics: &Metrics) -> Vec<String> {
    widgets
        .iter()
        .map(|widget| match widget {
            Widget::CpuPercent | Widget::CpuBar => percent(metrics.cpu_percent),
            Widget::MemBar => percent(memory_percent(metrics.memory)),
            Widget::CpuMemBar => format!(
                "{} {}",
                percent(metrics.cpu_percent),
                percent(memory_percent(metrics.memory))
            ),
            Widget::Temperature => temperature_text(metrics.temperature),
            Widget::DiskFree => free_text(metrics.disk),
            Widget::Text { value } => value.clone(),
        })
        .collect()
}

// Caption + value block, centered in a widget slot. Scales as `draw_line` takes them.
pub(super) const CAPTION_SCALE: u32 = 2;

/// Stacks the widgets top to bottom in the `stack` box, one equal slot per widget. Each
/// widget draws into its own slot, so text that is too tall is clipped there.
pub(super) fn draw_widgets(
    canvas: &mut RgbImage,
    widgets: &[Widget],
    metrics: &Metrics,
    bg: [u8; 3],
    fg: [u8; 3],
    stack: PixelRect,
    bar: Option<&BarStyle>,
) {
    // `widgets` is never empty here; `max(1)` only keeps the division safe.
    let count = widgets.len().max(1) as u32;
    let slot_h = stack.h / count;
    for (i, widget) in widgets.iter().enumerate() {
        let mut slot = ImageBuffer::from_pixel(stack.w, slot_h, Rgb(bg));
        draw_widget(&mut slot, widget, metrics, fg, bar);
        let y = stack.y + i as u32 * slot_h;
        imageops::replace(canvas, &slot, i64::from(stack.x), i64::from(y));
    }
}
