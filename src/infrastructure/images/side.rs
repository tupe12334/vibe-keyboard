use std::path::Path;

use image::{imageops, DynamicImage, ImageBuffer, Rgb, RgbImage};

use super::profile_key::source_image;
use super::text::draw_label;
use crate::domain::profile::{inner_rect, parse_color, PixelRect, SideConfig, SideInset};
use crate::infrastructure::system_metrics::Metrics;

mod cpu_bar;
mod value_text;
pub(super) mod widgets;

use widgets::draw_widgets;

#[cfg(test)]
use crate::domain::profile::Widget;
#[cfg(test)]
use crate::infrastructure::system_metrics::Disk;
#[cfg(test)]
#[cfg(test)]
use value_text::{free_text, percent, temperature_text};

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
            side.bar_style.as_ref(),
        );
    }
    canvas
}

#[cfg(test)]
#[path = "side_tests.rs"]
mod tests;
