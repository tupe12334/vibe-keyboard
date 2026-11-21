//! Paints TrueType glyph outlines with coverage blending. The font layout lives in the parent.

use std::sync::LazyLock;

use ab_glyph::{point, Font, FontRef, ScaleFont};
use image::{Rgb, RgbImage};

use crate::fonts::NOTO_SANS_HEBREW;

static FONT: LazyLock<Option<FontRef<'static>>> =
    LazyLock::new(|| FontRef::try_from_slice(NOTO_SANS_HEBREW).ok());

/// Advance width in pixels of `text` at em size `px`.
pub(super) fn run_width(text: &str, px: f32) -> f32 {
    let Some(font) = FONT.as_ref() else {
        return 0.0;
    };
    let scaled = font.as_scaled(px);
    text.chars()
        .map(|c| scaled.h_advance(font.glyph_id(c)))
        .sum()
}

/// Paints `text` (already in visual order) with its baseline at `baseline`, starting at `x`.
pub(super) fn draw_run(
    img: &mut RgbImage,
    x: f32,
    baseline: f32,
    text: &str,
    fg: Rgb<u8>,
    px: f32,
) {
    let Some(font) = FONT.as_ref() else {
        return;
    };
    let mut pen = x;
    for c in text.chars() {
        let id = font.glyph_id(c);
        let advance = font.as_scaled(px).h_advance(id);
        if let Some(outlined) =
            font.outline_glyph(id.with_scale_and_position(px, point(pen, baseline)))
        {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, cov| {
                let px_x = i64::from(bounds.min.x as i32) + i64::from(gx);
                let px_y = i64::from(bounds.min.y as i32) + i64::from(gy);
                blend(img, px_x, px_y, fg, cov);
            });
        }
        pen += advance;
    }
}

fn blend(img: &mut RgbImage, x: i64, y: i64, fg: Rgb<u8>, cov: f32) {
    let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
        return;
    };
    if x >= img.width() || y >= img.height() {
        return;
    }
    let px = img.get_pixel_mut(x, y);
    for (dst, src) in px.0.iter_mut().zip(fg.0) {
        let (d, s) = (f32::from(*dst), f32::from(src));
        *dst = (d + (s - d) * cov.clamp(0.0, 1.0)).round() as u8;
    }
}
