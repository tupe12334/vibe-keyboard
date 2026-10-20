use image::{Rgb, RgbImage};

mod glyphs;
mod raster;
mod wrap;

use glyphs::glyph;
use raster::draw_glyph;
use wrap::wrap;
// ponytail: fixed 3x5 pixel font, A-Z 0-9 and a few symbols, scaled 3x. Other characters draw
// blank. Add a real font crate (ab_glyph) when labels need lowercase or non-ASCII text.
const SCALE: u32 = 3;
const GLYPH_W: u32 = 3;
const GLYPH_H: u32 = 5;
const MAX_LINES: usize = 4;

/// Draw `text` centered on `img`, word-wrapped to as many lines as fit.
pub fn draw_label(img: &mut RgbImage, text: &str, fg: Rgb<u8>) {
    draw_wrapped(img, text, fg, MAX_LINES, SCALE);
}

/// Like `draw_label`, but at most three lines. Used for session titles under a status bar.
pub fn draw_title(img: &mut RgbImage, text: &str, fg: Rgb<u8>) {
    draw_wrapped(img, text, fg, 3, SCALE);
}

/// Like `draw_label`, with glyphs `scale` times the base size. Lines past the bottom are clipped.
pub fn draw_label_scaled(img: &mut RgbImage, text: &str, fg: Rgb<u8>, scale: u32) {
    draw_wrapped(img, text, fg, MAX_LINES, scale);
}

/// One line of small text (scale 1, 5 px tall) in the top-right corner of `img`. Fits a 12 px bar.
pub fn draw_tag(img: &mut RgbImage, text: &str, fg: Rgb<u8>) {
    let width = text.chars().count() as u32 * (GLYPH_W + 1);
    let mut x = img.width().saturating_sub(width + 2);
    for c in text.chars() {
        draw_glyph(img, x, 3, glyph(c), fg, 1);
        x += GLYPH_W + 1;
    }
}

fn draw_wrapped(img: &mut RgbImage, text: &str, fg: Rgb<u8>, max_lines: usize, scale: u32) {
    let advance = (GLYPH_W + 1) * scale;
    let line_h = (GLYPH_H + 1) * scale;
    let per_line = (img.width() / advance) as usize;
    let lines = wrap(text, per_line, max_lines);
    let total = lines.len() as u32 * line_h;
    let top = img.height().saturating_sub(total) / 2;
    for (row, line) in lines.iter().enumerate() {
        let width = line.chars().count() as u32 * advance;
        let left = img.width().saturating_sub(width) / 2;
        let y = top + row as u32 * line_h;
        for (i, c) in line.chars().enumerate() {
            draw_glyph(img, left + i as u32 * advance, y, glyph(c), fg, scale);
        }
    }
}

/// Height in pixels of one line of text at `scale`.
pub const fn text_height(scale: u32) -> u32 {
    GLYPH_H * scale
}

/// Draw `text` on one line, centered horizontally, with its top edge at `y`. Pixels past
/// the image edge are dropped.
pub fn draw_line(img: &mut RgbImage, y: u32, text: &str, fg: Rgb<u8>, scale: u32) {
    draw_line_within(img, 0, img.width(), y, text, fg, scale);
}

/// Like `draw_line`, but centered in the columns `x..x + width` only.
pub fn draw_line_within(
    img: &mut RgbImage,
    x: u32,
    width: u32,
    y: u32,
    text: &str,
    fg: Rgb<u8>,
    scale: u32,
) {
    let advance = (GLYPH_W + 1) * scale;
    let text_w = text.chars().count() as u32 * advance;
    let left = x + width.saturating_sub(text_w) / 2;
    for (i, c) in text.chars().enumerate() {
        draw_glyph(img, left + i as u32 * advance, y, glyph(c), fg, scale);
    }
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
