use image::{ImageBuffer, Rgb, RgbImage};

use super::draw::{draw_thick_line, fill_circle};

pub fn generate_centy_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);

    let bg = Rgb([15u8, 15, 15]);
    let accent = Rgb([99u8, 102, 241]); // indigo
    let white = Rgb([240u8, 240, 240]);

    for p in img.pixels_mut() {
        *p = bg;
    }

    // Outer circle
    for angle_deg in 0..360u32 {
        let angle = (angle_deg as f64) * std::f64::consts::PI / 180.0;
        for r in 36u32..=42 {
            let x = (50.0 + (r as f64) * angle.cos()).round() as i32;
            let y = (50.0 + (r as f64) * angle.sin()).round() as i32;
            if x >= 0 && x < 100 && y >= 0 && y < 100 {
                img.put_pixel(x as u32, y as u32, accent);
            }
        }
    }

    // "C" arc — draw from ~45° to ~315° (leaving a gap on the right)
    for angle_deg in 50..=310u32 {
        let angle = (angle_deg as f64) * std::f64::consts::PI / 180.0;
        for r in 20u32..=28 {
            let x = (50.0 + (r as f64) * angle.cos()).round() as i32;
            let y = (50.0 + (r as f64) * angle.sin()).round() as i32;
            if x >= 0 && x < 100 && y >= 0 && y < 100 {
                img.put_pixel(x as u32, y as u32, white);
            }
        }
    }

    // Top terminal of C
    draw_thick_line(&mut img, 50, 22, 62, 22, white, 3);
    // Bottom terminal of C
    draw_thick_line(&mut img, 50, 78, 62, 78, white, 3);

    // Small dots in corner for "list" metaphor
    fill_circle(&mut img, 80, 20, 3, accent);
    fill_circle(&mut img, 80, 30, 3, accent);
    fill_circle(&mut img, 80, 40, 3, accent);

    img
}
