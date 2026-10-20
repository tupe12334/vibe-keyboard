use image::{ImageBuffer, Rgb, RgbImage};

use super::fill_rect;

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
