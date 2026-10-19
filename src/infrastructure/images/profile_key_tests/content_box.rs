//! Bounding box of the pixels that differ from a background color, for key tests.

use image::RgbImage;

/// Bounding box `(x, y, w, h)` of the pixels that differ from `bg`.
pub(super) fn content_bbox(img: &RgbImage, bg: [u8; 3]) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p.0 != bg {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    (x0 != u32::MAX).then(|| (x0, y0, x1 - x0 + 1, y1 - y0 + 1))
}
