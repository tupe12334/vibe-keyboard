//! Real-font text for characters the 3x5 pixel font cannot draw (for example Hebrew).

use image::{Rgb, RgbImage};
use unicode_bidi::BidiInfo;

use super::wrap::wrap;
use super::GLYPH_H;

mod outline;
use outline::{draw_run, run_width};

/// Em size in pixels per unit of text scale. Scale 3 gives 21 px, near the pixel font cap height.
const PX_PER_SCALE: f32 = 7.0;
/// Baseline position as a fraction of the em, measured from the top of a text row.
const BASELINE: f32 = 0.75;

fn em(scale: u32) -> f32 {
    scale as f32 * PX_PER_SCALE
}

/// Reorders one logical line into visual (left-to-right) order.
pub(super) fn visual_order(line: &str) -> String {
    let info = BidiInfo::new(line, None);
    match info.paragraphs.first() {
        Some(para) => info.reorder_line(para, para.range.clone()).into_owned(),
        None => line.to_string(),
    }
}

/// Draws `text` centered in the columns `x..x + width`, with its row top at `y`.
pub(super) fn draw_centered_within(
    img: &mut RgbImage,
    x: u32,
    width: u32,
    y: u32,
    text: &str,
    fg: Rgb<u8>,
    scale: u32,
) {
    let px = em(scale);
    let visual = visual_order(text);
    let left = x as f32 + (width as f32 - run_width(&visual, px)) / 2.0;
    draw_run(img, left, y as f32 + px * BASELINE, &visual, fg, px);
}

/// Word-wraps `text` to the image width and centers the block vertically.
pub(super) fn draw_wrapped(
    img: &mut RgbImage,
    text: &str,
    fg: Rgb<u8>,
    max_lines: usize,
    scale: u32,
) {
    let px = em(scale);
    let line_h = (GLYPH_H + 1) * scale;
    let chars = text.chars().count().max(1) as f32;
    let avg = (run_width(text, px) / chars).max(1.0);
    let per_line = (img.width() as f32 / avg).max(1.0) as usize;
    let lines = wrap(text, per_line, max_lines);
    let total = lines.len() as u32 * line_h;
    let top = img.height().saturating_sub(total) / 2;
    for (row, line) in lines.iter().enumerate() {
        let y = top + row as u32 * line_h;
        draw_centered_within(img, 0, img.width(), y, line, fg, scale);
    }
}

/// Draws `text` right-aligned in the top-right corner, one line at scale 1.
pub(super) fn draw_tag(img: &mut RgbImage, text: &str, fg: Rgb<u8>) {
    let px = em(1);
    let visual = visual_order(text);
    let x = (img.width() as f32 - run_width(&visual, px) - 2.0).max(0.0);
    draw_run(img, x, 3.0 + px * BASELINE, &visual, fg, px);
}
