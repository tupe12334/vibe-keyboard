use std::path::Path;

use image::{imageops, ImageBuffer, Rgb, RgbImage};

use super::profile_key::source_image;
use super::text::{draw_label, draw_label_scaled, draw_line};
use crate::domain::profile::{parse_color, SideConfig, Widget};
use crate::infrastructure::system_metrics::{Disk, Metrics};

/// Side screen size in pixels. Never send a taller image: it spills onto keys 5, 10 and 15.
pub const SIDE_WIDTH: u32 = 480;
pub const SIDE_HEIGHT: u32 = 100;
/// Visible part of the side screen, as x offset and width. Keep content inside it.
pub const SAFE_X: u32 = 30;
pub const SAFE_WIDTH: u32 = 410;

const FALLBACK_BG: [u8; 3] = [20, 20, 20];
const FALLBACK_FG: [u8; 3] = [230, 230, 230];
const BLANK: [u8; 3] = [0, 0, 0];
const TRACK: [u8; 3] = [60, 60, 60];
const GREEN: [u8; 3] = [46, 160, 67];
const AMBER: [u8; 3] = [230, 160, 30];
const RED: [u8; 3] = [220, 60, 50];

// Widget layout inside one slot, in pixels from the slot top.
const CAPTION_Y: u32 = 8;
const CAPTION_SCALE: u32 = 2;
const VALUE_Y: u32 = 36;
const VALUE_SCALE: u32 = 5;
const BAR_Y: u32 = 36;
const BAR_H: u32 = 22;
const BAR_PAD: u32 = 8;
const BAR_VALUE_Y: u32 = 68;
const BAR_VALUE_SCALE: u32 = 3;

/// The bitmap the side screen shows. The controller sends this image, and the editor
/// previews this same image. `None` gives a black strip. `metrics` feeds the widgets.
pub fn side_image(side: Option<&SideConfig>, root: &Path, metrics: &Metrics) -> RgbImage {
    let Some(side) = side else {
        return ImageBuffer::from_pixel(SIDE_WIDTH, SIDE_HEIGHT, Rgb(BLANK));
    };
    let bg = parse_color(&side.bg).unwrap_or(FALLBACK_BG);
    let fg = parse_color(&side.fg).unwrap_or(FALLBACK_FG);
    let mut canvas = ImageBuffer::from_pixel(SIDE_WIDTH, SIDE_HEIGHT, Rgb(bg));
    // Content is drawn in the safe box, then placed at its x offset.
    let mut safe = ImageBuffer::from_pixel(SAFE_WIDTH, SIDE_HEIGHT, Rgb(bg));
    if side.widgets.is_empty() {
        let picture = side
            .image
            .as_deref()
            .and_then(|image| source_image(image, root, SIDE_HEIGHT));
        match picture {
            Some(img) => {
                let left = (SAFE_WIDTH - SIDE_HEIGHT) / 2;
                imageops::replace(&mut safe, &img, i64::from(left), 0);
            }
            None => draw_label(&mut safe, &side.label, Rgb(fg)),
        }
    } else {
        draw_widgets(&mut safe, &side.widgets, metrics, bg, fg);
    }
    imageops::replace(&mut canvas, &safe, i64::from(SAFE_X), 0);
    canvas
}

/// Splits the safe box into equal slots, one per widget, left to right. Each widget draws
/// into its own slot, so text that is too wide is clipped there.
fn draw_widgets(
    safe: &mut RgbImage,
    widgets: &[Widget],
    metrics: &Metrics,
    bg: [u8; 3],
    fg: [u8; 3],
) {
    // `widgets` is never empty here; `max(1)` only keeps the division safe.
    let count = widgets.len().max(1) as u32;
    let slot_w = SAFE_WIDTH / count;
    for (i, widget) in widgets.iter().enumerate() {
        let mut slot = ImageBuffer::from_pixel(slot_w, SIDE_HEIGHT, Rgb(bg));
        draw_widget(&mut slot, widget, metrics, fg);
        imageops::replace(safe, &slot, i64::from(i as u32 * slot_w), 0);
    }
}

fn draw_widget(slot: &mut RgbImage, widget: &Widget, metrics: &Metrics, fg: [u8; 3]) {
    let fg = Rgb(fg);
    match widget {
        Widget::CpuPercent => {
            draw_line(
                slot,
                VALUE_Y,
                &percent(metrics.cpu_percent),
                fg,
                VALUE_SCALE,
            );
        }
        Widget::CpuBar => {
            draw_line(slot, CAPTION_Y, "CPU", fg, CAPTION_SCALE);
            draw_bar(slot, metrics.cpu_percent);
            let text = percent(metrics.cpu_percent);
            draw_line(slot, BAR_VALUE_Y, &text, fg, BAR_VALUE_SCALE);
        }
        Widget::Temperature => {
            draw_line(slot, CAPTION_Y, "TEMP", fg, CAPTION_SCALE);
            let text = temperature_text(metrics.temperature);
            draw_line(slot, VALUE_Y, &text, fg, VALUE_SCALE);
        }
        Widget::DiskFree => {
            draw_line(slot, CAPTION_Y, "DISK", fg, CAPTION_SCALE);
            let text = free_text(metrics.disk);
            draw_line(slot, VALUE_Y, &text, fg, VALUE_SCALE);
        }
        Widget::Text { value } => draw_label_scaled(slot, value, fg, VALUE_SCALE),
    }
}

/// Track, then the filled part in the threshold color.
fn draw_bar(slot: &mut RgbImage, percent: u8) {
    let percent = percent.min(100);
    let x0 = BAR_PAD;
    let width = slot.width().saturating_sub(2 * BAR_PAD);
    fill_rect(slot, x0, BAR_Y, width, BAR_H, TRACK);
    let filled = width * u32::from(percent) / 100;
    fill_rect(slot, x0, BAR_Y, filled, BAR_H, bar_color(percent));
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
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn all_widgets() -> Vec<Widget> {
        vec![
            Widget::CpuBar,
            Widget::CpuPercent,
            Widget::Temperature,
            Widget::DiskFree,
            Widget::Text {
                value: "BUILD OK".into(),
            },
        ]
    }

    fn sample() -> Metrics {
        Metrics {
            cpu_percent: 87,
            disk: Some(Disk {
                free: 412_000_000_000,
                total: 994_000_000_000,
            }),
            temperature: Some(71.4),
        }
    }

    #[test]
    fn side_image_is_480_by_100() {
        let root = PathBuf::from("/nonexistent");
        let metrics = Metrics::default();
        assert_eq!(side_image(None, &root, &metrics).dimensions(), (480, 100));
        let side = SideConfig {
            label: "Build ok".into(),
            ..SideConfig::default()
        };
        assert_eq!(
            side_image(Some(&side), &root, &metrics).dimensions(),
            (480, 100)
        );
    }

    #[test]
    fn label_stays_inside_the_safe_area() {
        let root = PathBuf::from("/nonexistent");
        let side = SideConfig {
            label: "LONG LABEL THAT WRAPS ACROSS LINES".into(),
            bg: "#000000".into(),
            fg: "#FFFFFF".into(),
            image: None,
            widgets: Vec::new(),
        };
        let img = side_image(Some(&side), &root, &Metrics::default());
        let inside = |x: u32| (SAFE_X..SAFE_X + SAFE_WIDTH).contains(&x);
        for (x, _, pixel) in img.enumerate_pixels() {
            if pixel.0 == [255, 255, 255] {
                assert!(
                    inside(x),
                    "foreground pixel at x={x} is outside the safe area"
                );
            }
        }
    }

    #[test]
    fn widget_layout_fits_the_safe_area_for_any_count() {
        let root = PathBuf::from("/nonexistent");
        for count in 1..=5 {
            let side = SideConfig {
                bg: "#000000".into(),
                fg: "#FFFFFF".into(),
                widgets: all_widgets().into_iter().cycle().take(count).collect(),
                ..SideConfig::default()
            };
            let img = side_image(Some(&side), &root, &sample());
            // Everything outside the safe box is background, so nothing spills past it.
            for (x, _, pixel) in img.enumerate_pixels() {
                if !(SAFE_X..SAFE_X + SAFE_WIDTH).contains(&x) {
                    assert_eq!(pixel.0, [0, 0, 0], "x={x} count={count}");
                }
            }
            assert!(img.pixels().any(|p| p.0 == [255, 255, 255]));
        }
    }

    #[test]
    fn widgets_replace_label_and_image() {
        let root = PathBuf::from("/nonexistent");
        let metrics = sample();
        let plain = SideConfig {
            label: "HELLO".into(),
            ..SideConfig::default()
        };
        let with_widgets = SideConfig {
            widgets: vec![Widget::CpuPercent],
            ..plain.clone()
        };
        assert_ne!(
            side_image(Some(&plain), &root, &metrics),
            side_image(Some(&with_widgets), &root, &metrics)
        );
    }

    #[test]
    fn bar_color_thresholds() {
        assert_eq!(bar_color(0), GREEN);
        assert_eq!(bar_color(59), GREEN);
        assert_eq!(bar_color(60), AMBER);
        assert_eq!(bar_color(84), AMBER);
        assert_eq!(bar_color(85), RED);
        assert_eq!(bar_color(100), RED);
    }

    #[test]
    fn bar_fill_follows_the_percent() {
        let root = PathBuf::from("/nonexistent");
        let side = SideConfig {
            widgets: vec![Widget::CpuBar],
            ..SideConfig::default()
        };
        let empty = side_image(Some(&side), &root, &Metrics::default());
        let full = side_image(
            Some(&side),
            &root,
            &Metrics {
                cpu_percent: 100,
                ..Metrics::default()
            },
        );
        let count = |img: &RgbImage, color: [u8; 3]| img.pixels().filter(|p| p.0 == color).count();
        assert_eq!(
            count(&empty, GREEN) + count(&empty, AMBER) + count(&empty, RED),
            0
        );
        assert!(count(&full, RED) > 300, "full bar too small");
    }

    #[test]
    fn value_text_formats() {
        assert_eq!(percent(7), "7%");
        assert_eq!(temperature_text(Some(71.4)), "71C");
        assert_eq!(temperature_text(None), "N/A");
        assert_eq!(free_text(None), "N/A");
        assert_eq!(
            free_text(Some(Disk {
                free: 412_000_000_000,
                total: 1
            })),
            "412G"
        );
        assert_eq!(
            free_text(Some(Disk {
                free: 1_200_000_000_000,
                total: 1
            })),
            "1.2T"
        );
    }
}
