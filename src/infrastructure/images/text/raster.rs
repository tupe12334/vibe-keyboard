//! Paints the pixels of one glyph. The metrics come from the parent text module.

use image::{Rgb, RgbImage};

use super::GLYPH_W;

pub(super) fn draw_glyph(
    img: &mut RgbImage,
    x: u32,
    y: u32,
    rows: [u8; 5],
    fg: Rgb<u8>,
    scale: u32,
) {
    for (ry, bits) in rows.iter().enumerate() {
        for rx in 0..GLYPH_W {
            if bits & (1 << (GLYPH_W - 1 - rx)) == 0 {
                continue;
            }
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = x + rx * scale + dx;
                    let py = y + ry as u32 * scale + dy;
                    if px < img.width() && py < img.height() {
                        img.put_pixel(px, py, fg);
                    }
                }
            }
        }
    }
}
