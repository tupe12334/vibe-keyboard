use image::{ImageBuffer, Rgb, RgbImage};

use super::draw::{draw_thick_line, fill_circle};

pub fn generate_log_file_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);

    let bg = Rgb([20u8, 20, 20]);
    let green = Rgb([80u8, 200, 80]);
    let white = Rgb([230u8, 230, 230]);
    let gray = Rgb([130u8, 130, 130]);

    for p in img.pixels_mut() {
        *p = bg;
    }

    // Document body
    for y in 15u32..=85 {
        for x in 25u32..=75 {
            img.put_pixel(x, y, white);
        }
    }

    // Folded top-right corner
    for y in 15u32..=30 {
        for x in 60u32..=75 {
            img.put_pixel(x, y, bg);
        }
    }
    draw_thick_line(&mut img, 60, 15, 75, 30, gray, 2);
    draw_thick_line(&mut img, 60, 15, 60, 30, gray, 1);
    draw_thick_line(&mut img, 60, 30, 75, 30, gray, 1);

    // Text lines on document
    for y in [38u32, 48, 58, 68] {
        for x in 33u32..=67 {
            img.put_pixel(x, y, gray);
        }
    }

    // Green circle badge (log indicator)
    fill_circle(&mut img, 72, 72, 14, bg);
    fill_circle(&mut img, 72, 72, 12, green);

    // ">" arrow inside badge
    draw_thick_line(&mut img, 67, 67, 74, 72, bg, 2);
    draw_thick_line(&mut img, 67, 77, 74, 72, bg, 2);

    img
}
