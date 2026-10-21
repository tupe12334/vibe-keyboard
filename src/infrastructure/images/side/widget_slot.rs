use image::{Rgb, RgbImage};

use super::super::text::{draw_label_scaled, draw_line, text_height};
use super::bars::{draw_percent_bar, draw_split};
use super::cpu_bar::BarLook;
use super::value_text::{free_text, percent, temperature_text};
use super::widgets::CAPTION_SCALE;
use crate::domain::profile::{BarStyle, Widget};
use crate::infrastructure::system_metrics::memory_percent;
use crate::infrastructure::system_metrics::Metrics;

const VALUE_SCALE: u32 = 5;
const BLOCK_GAP: u32 = 4;

pub(super) fn draw_widget(
    slot: &mut RgbImage,
    widget: &Widget,
    metrics: &Metrics,
    fg: [u8; 3],
    bar: Option<&BarStyle>,
) {
    let fg = Rgb(fg);
    match widget {
        Widget::CpuPercent => caption_value(slot, "CPU", &percent(metrics.cpu_percent), fg),
        Widget::CpuBar => {
            let look = BarLook::resolve(bar, fg.0);
            draw_percent_bar(slot, "CPU", metrics.cpu_percent, &look, fg);
        }
        Widget::MemBar => {
            let look = BarLook::resolve(bar, fg.0);
            draw_percent_bar(slot, "MEM", memory_percent(metrics.memory), &look, fg);
        }
        Widget::CpuMemBar => {
            let look = BarLook::resolve(bar, fg.0);
            draw_split(
                slot,
                metrics.cpu_percent,
                memory_percent(metrics.memory),
                &look,
            );
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
