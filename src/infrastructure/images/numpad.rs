use image::{ImageBuffer, Rgb, RgbImage};

/// Draw a filled rectangle on the image.
fn fill_rect(img: &mut RgbImage, x0: u32, y0: u32, x1: u32, y1: u32, color: Rgb<u8>) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            if x < 100 && y < 100 {
                img.put_pixel(x, y, color);
            }
        }
    }
}

/// Return (top, `top_left`, `top_right`, middle, `bot_left`, `bot_right`, bottom)
/// for a 7-segment digit 1–9.
const fn segments(digit: u8) -> (bool, bool, bool, bool, bool, bool, bool) {
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

/// Generate a 100×100 "CLR" (clear) button image — red background with white X.
pub fn generate_numpad_clear_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);
    let bg = Rgb([15u8, 15, 15]);
    let red = Rgb([200u8, 50, 50]);
    let white = Rgb([220u8, 220, 220]);

    for p in img.pixels_mut() {
        *p = bg;
    }

    // Red rounded square background
    fill_rect(&mut img, 20, 20, 80, 80, red);

    // White X (two thick diagonal lines)
    for i in -3i32..=3 {
        for j in 0i32..=44 {
            let t = j as u32;
            // Top-left to bottom-right diagonal
            let x = (30 + j + i) as u32;
            let y = (30 + j) as u32;
            if x < 100 && y < 100 {
                img.put_pixel(x, y, white);
            }
            // Top-right to bottom-left diagonal
            let x2 = (70i32 - j + i) as u32;
            if x2 < 100 && y < 100 {
                img.put_pixel(x2, y, white);
            }
            let _ = t;
        }
    }

    img
}

/// Generate a 100×100 backspace button image — dark background with left-arrow.
pub fn generate_numpad_backspace_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);
    let bg = Rgb([15u8, 15, 15]);
    let accent = Rgb([80u8, 140, 200]);

    for p in img.pixels_mut() {
        *p = bg;
    }

    // Arrow shaft (horizontal bar)
    fill_rect(&mut img, 42, 46, 78, 54, accent);

    // Arrowhead (filled triangle pointing left)
    for dy in 0i32..=20 {
        let x_start = (22 + dy) as u32;
        let x_end = 42_u32;
        let y = (50 - 20 + dy) as u32;
        if y < 100 {
            fill_rect(&mut img, x_start, y, x_end, y, accent);
        }
        let y2 = (50 + 20 - dy) as u32;
        if y2 < 100 && y2 != y {
            fill_rect(&mut img, x_start, y2, x_end, y2, accent);
        }
    }

    img
}

/// Generate a 100×100 "numpad entry" icon for the main-page button.
pub fn generate_numpad_entry_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);
    let bg = Rgb([15u8, 15, 15]);
    let fg = Rgb([100u8, 200, 100]);

    for p in img.pixels_mut() {
        *p = bg;
    }

    // Draw a simple grid of 9 small squares representing a numpad
    let offsets: [(u32, u32); 9] = [
        (15, 15),
        (42, 15),
        (69, 15),
        (15, 42),
        (42, 42),
        (69, 42),
        (15, 69),
        (42, 69),
        (69, 69),
    ];
    for (ox, oy) in offsets {
        fill_rect(&mut img, ox, oy, ox + 16, oy + 16, fg);
    }

    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_numpad_digit_image_valid_size() {
        for d in 1u8..=9 {
            let img = generate_numpad_digit_image(d);
            assert_eq!(img.width(), 100);
            assert_eq!(img.height(), 100);
        }
    }

    #[test]
    fn generate_numpad_digit_image_has_foreground_pixels() {
        // digit 8 lights all segments — must have non-bg pixels
        let img = generate_numpad_digit_image(8);
        let fg = Rgb([220u8, 220, 220]);
        assert!(img.pixels().any(|p| *p == fg));
    }

    #[test]
    fn generate_numpad_clear_image_valid_size() {
        let img = generate_numpad_clear_image();
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 100);
    }

    #[test]
    fn generate_numpad_backspace_image_valid_size() {
        let img = generate_numpad_backspace_image();
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 100);
    }

    #[test]
    fn generate_numpad_entry_image_valid_size() {
        let img = generate_numpad_entry_image();
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 100);
    }

    #[test]
    fn segments_digit_1_only_right_verticals() {
        let (t, tl, tr, m, bl, br, b) = segments(1);
        assert!(!t && !tl && tr && !m && !bl && br && !b);
    }

    #[test]
    fn segments_digit_8_all_on() {
        let (t, tl, tr, m, bl, br, b) = segments(8);
        assert!(t && tl && tr && m && bl && br && b);
    }

    #[test]
    fn segments_unknown_digit_all_off() {
        let (t, tl, tr, m, bl, br, b) = segments(0);
        assert!(!t && !tl && !tr && !m && !bl && !br && !b);
    }
}
