use image::{ImageBuffer, Rgb, RgbImage};

use super::fill_rect;

/// Return (top, `top_left`, `top_right`, middle, `bot_left`, `bot_right`, bottom)
/// for a 7-segment digit 1–9.
pub(super) const fn segments(digit: u8) -> (bool, bool, bool, bool, bool, bool, bool) {
    match digit {
        1 => (false, false, true, false, false, true, false),
        2 => (true, false, true, true, true, false, true),
        3 => (true, false, true, true, false, true, true),
        4 => (false, true, true, true, false, true, false),
        5 => (true, true, false, true, false, true, true),
        6 => (true, true, false, true, true, true, true),
        7 => (true, false, true, false, false, true, false),
        8 => (true, true, true, true, true, true, true),
        9 => (true, true, true, true, false, true, true),
        _ => (false, false, false, false, false, false, false),
    }
}

/// Generate a 100×100 image showing a single numpad digit (1–9).
pub fn generate_numpad_digit_image(digit: u8) -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);
    let bg = Rgb([15u8, 15, 15]);
    let fg = Rgb([220u8, 220, 220]);

    for p in img.pixels_mut() {
        *p = bg;
    }

    let (t, tl, tr, m, bl, br, b) = segments(digit);

    // Horizontal segments (y range ±2 from center)
    if t {
        fill_rect(&mut img, 28, 16, 70, 22, fg); // top
    }
    if m {
        fill_rect(&mut img, 28, 47, 70, 53, fg); // middle
    }
    if b {
        fill_rect(&mut img, 28, 78, 70, 84, fg); // bottom
    }

    // Vertical segments (x range ±2 from edge)
    if tl {
        fill_rect(&mut img, 24, 24, 30, 45, fg); // top-left
    }
    if tr {
        fill_rect(&mut img, 68, 24, 74, 45, fg); // top-right
    }
    if bl {
        fill_rect(&mut img, 24, 55, 30, 76, fg); // bottom-left
    }
    if br {
        fill_rect(&mut img, 68, 55, 74, 76, fg); // bottom-right
    }

    img
}
