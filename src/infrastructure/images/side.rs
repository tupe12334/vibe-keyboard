use std::path::Path;

use image::{imageops, DynamicImage, ImageBuffer, Rgb, RgbImage};

use super::profile_key::source_image;
use super::text::{draw_label, draw_label_scaled, draw_line, text_height};
use crate::domain::profile::{inner_rect, parse_color, PixelRect, SideConfig, SideInset, Widget};
use crate::infrastructure::system_metrics::{Disk, Metrics};

/// Side screen render size, portrait: the panel is a vertical column. The upload rotates
/// it 90° counter-clockwise (`usb::side_upload`). Never render taller than this.
pub const SIDE_WIDTH: u32 = 100;
pub const SIDE_HEIGHT: u32 = 480;
/// Gap between the widget stack and the edges of the visible rect.
const MARGIN: u32 = 4;

/// The visible part of the column after the side insets. Content goes inside it.
pub fn safe_rect(inset: &SideInset) -> PixelRect {
    inner_rect(SIDE_WIDTH, SIDE_HEIGHT, inset.top, inset.bottom, 0, 0)
}

/// Widget stack: the visible rect less `MARGIN` on each side.
const fn stack_rect(safe: PixelRect) -> PixelRect {
    PixelRect {
        x: safe.x + MARGIN,
        y: safe.y + MARGIN,
        w: safe.w.saturating_sub(2 * MARGIN),
        h: safe.h.saturating_sub(2 * MARGIN),
    }
}

const FALLBACK_BG: [u8; 3] = [20, 20, 20];
const FALLBACK_FG: [u8; 3] = [230, 230, 230];
const BLANK: [u8; 3] = [0, 0, 0];
const TRACK: [u8; 3] = [60, 60, 60];
const GREEN: [u8; 3] = [46, 160, 67];
const AMBER: [u8; 3] = [230, 160, 30];
const RED: [u8; 3] = [220, 60, 50];

// Caption + value block, centered in a widget slot. Scales as `draw_line` takes them.
const CAPTION_SCALE: u32 = 2;
const VALUE_SCALE: u32 = 5;
const BLOCK_GAP: u32 = 4;
// Vertical CPU bar: caption on top, bar below, percent text at the bottom.
const BAR_X: u32 = 30;
const BAR_W: u32 = 40;
const BAR_VALUE_SCALE: u32 = 3;

/// The bitmap the side screen shows, portrait `SIDE_WIDTH` x `SIDE_HEIGHT`. The editor
/// previews this image as is. The controller rotates it for upload. `None` gives a black
/// column. `metrics` feeds the widgets. The inset band keeps the background color.
pub fn side_image(
    side: Option<&SideConfig>,
    root: &Path,
    metrics: &Metrics,
    inset: &SideInset,
) -> RgbImage {
    let Some(side) = side else {
        return ImageBuffer::from_pixel(SIDE_WIDTH, SIDE_HEIGHT, Rgb(BLANK));
    };
    let bg = parse_color(&side.bg).unwrap_or(FALLBACK_BG);
    let fg = parse_color(&side.fg).unwrap_or(FALLBACK_FG);
    let mut canvas = ImageBuffer::from_pixel(SIDE_WIDTH, SIDE_HEIGHT, Rgb(bg));
    let safe = safe_rect(inset);
    if safe.w == 0 || safe.h == 0 {
        return canvas;
    }
    if side.widgets.is_empty() {
        let picture = side
            .image
            .as_deref()
            .and_then(|image| source_image(image, root, safe.h));
        if let Some(img) = picture {
            let fill = DynamicImage::ImageRgb8(img)
                .resize_to_fill(safe.w, safe.h, image::imageops::FilterType::Triangle)
                .to_rgb8();
            imageops::replace(&mut canvas, &fill, i64::from(safe.x), i64::from(safe.y));
        } else {
            let mut area = ImageBuffer::from_pixel(safe.w, safe.h, Rgb(bg));
            draw_label(&mut area, &side.label, Rgb(fg));
            imageops::replace(&mut canvas, &area, i64::from(safe.x), i64::from(safe.y));
        }
    } else {
        draw_widgets(
            &mut canvas,
            &side.widgets,
            metrics,
            bg,
            fg,
            stack_rect(safe),
        );
    }
    canvas
}

/// Stacks the widgets top to bottom in the `stack` box, one equal slot per widget. Each
/// widget draws into its own slot, so text that is too tall is clipped there.
fn draw_widgets(
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

/// Gap between the bar and the slot edges, and between the bar and its text.
const BAR_PAD: u32 = 4;

/// Rows `(top, bottom)` of the CPU bar in a slot of `slot_h`, bottom exclusive. The caption
/// sits above it and the percent text below it.
fn bar_span(slot_h: u32) -> (u32, u32) {
    let top = text_height(CAPTION_SCALE) + 2 * BAR_PAD;
    let bottom = slot_h.saturating_sub(2 * BAR_PAD + text_height(BAR_VALUE_SCALE));
    (top, bottom.max(top))
}

/// Track, then the filled part in the threshold color. The fill rises from the bottom.
fn draw_bar(slot: &mut RgbImage, percent: u8) {
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
pub const fn bar_color(percent: u8) -> [u8; 3] {
    match percent {
        0..=59 => GREEN,
        60..=84 => AMBER,
        _ => RED,
    }
}

fn percent(value: u8) -> String {
    format!("{value}%")
}

fn temperature_text(celsius: Option<f32>) -> String {
    celsius.map_or_else(|| "N/A".to_string(), |t| format!("{t:.0}C"))
}

fn free_text(disk: Option<Disk>) -> String {
    disk.map_or_else(|| "N/A".to_string(), |d| free_space(d.free))
}

/// Decimal units, as Finder shows them: `1.2T` from a terabyte, else whole `G`.
fn free_space(bytes: u64) -> String {
    const GB: f64 = 1e9;
    const TB: f64 = 1e12;
    let value = bytes as f64;
    if value >= TB {
        format!("{:.1}T", value / TB)
    } else {
        format!("{:.0}G", value / GB)
    }
}

#[cfg(test)]
#[path = "side_tests.rs"]
mod tests;
