//! Fits a key picture into the visible rect: trims its uniform border, then scales it.

use image::imageops::{self, FilterType};
use image::{ImageBuffer, Rgb, RgbImage};

use crate::domain::profile::{inner_rect, KeyInset};
use crate::infrastructure::usb::DECK_IMAGE_SIZE;

/// Gap in deck px between the content and the visible rect's edge.
const FILL_MARGIN: u32 = 2;
/// A pixel counts as content when some channel differs from the border color by more than this.
const TRIM_TOLERANCE: u8 = 24;

pub(super) fn fit_into_key(
    picture: &RgbImage,
    inset: &KeyInset,
    bg: [u8; 3],
    trim: bool,
) -> RgbImage {
    let size = DECK_IMAGE_SIZE;
    let mut canvas = ImageBuffer::from_pixel(size, size, Rgb(bg));
    let rect = inner_rect(size, size, inset.top, inset.bottom, inset.left, inset.right);
    let margin = FILL_MARGIN.min(rect.w.min(rect.h) / 4);
    let box_w = rect.w - 2 * margin;
    let box_h = rect.h - 2 * margin;
    if box_w == 0 || box_h == 0 {
        return canvas;
    }
    let cropped = if trim { trim_border(picture) } else { None };
    let content = cropped.as_ref().unwrap_or(picture);
    let (cw, ch) = content.dimensions();
    let scale = f64::min(
        f64::from(box_w) / f64::from(cw),
        f64::from(box_h) / f64::from(ch),
    );
    let w = ((f64::from(cw) * scale).round() as u32).clamp(1, box_w);
    let h = ((f64::from(ch) * scale).round() as u32).clamp(1, box_h);
    let fitted = imageops::resize(content, w, h, FilterType::Triangle);
    let x = rect.x + margin + (box_w - w) / 2;
    let y = rect.y + margin + (box_h - h) / 2;
    imageops::replace(&mut canvas, &fitted, i64::from(x), i64::from(y));
    canvas
}

/// The box that holds every pixel unlike the picture's top-left corner. `None` when the
/// picture is one color, so there is nothing to trim.
fn trim_border(picture: &RgbImage) -> Option<RgbImage> {
    let (w, h) = picture.dimensions();
    if w == 0 || h == 0 {
        return None;
    }
    let border = picture.get_pixel(0, 0).0;
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for (x, y, p) in picture.enumerate_pixels() {
        let gap =
            p.0.iter()
                .zip(border)
                .map(|(a, b)| a.abs_diff(b))
                .max()
                .unwrap_or(0);
        if gap > TRIM_TOLERANCE {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    if x1 < x0 || y1 < y0 {
        return None;
    }
    Some(imageops::crop_imm(picture, x0, y0, x1 - x0 + 1, y1 - y0 + 1).to_image())
}
