use image::{ImageBuffer, Rgb, RgbImage};

use super::draw::{draw_thick_line, fill_circle};

pub fn generate_claude_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);

    let bg = Rgb([20u8, 20, 20]);
    let orange = Rgb([218u8, 120, 60]);
    let light = Rgb([240u8, 160, 90]);

    for p in img.pixels_mut() {
        *p = bg;
    }

    // Draw a large arc "C" shape using thick line segments approximating a circle arc
    let cx = 50i32;
    let cy = 50i32;
    let r = 30i32;

    // Draw arc from ~40° to ~320° (leaving a gap on the right side)
    let start_deg: i32 = 45;
    let end_deg: i32 = 315;
    let steps = 60;
    let mut prev_x = cx + (r as f64 * (start_deg as f64).to_radians().cos()) as i32;
    let mut prev_y = cy + (r as f64 * (start_deg as f64).to_radians().sin()) as i32;

    for i in 1..=steps {
        let angle = start_deg as f64 + (end_deg - start_deg) as f64 * i as f64 / steps as f64;
        let nx = cx + (r as f64 * angle.to_radians().cos()) as i32;
        let ny = cy + (r as f64 * angle.to_radians().sin()) as i32;
        draw_thick_line(&mut img, prev_x, prev_y, nx, ny, orange, 6);
        prev_x = nx;
        prev_y = ny;
    }

    // Small dot accent (top-right of the C)
    let dot_x = cx + (r as f64 * (30f64).to_radians().cos()) as i32;
    let dot_y = cy + (r as f64 * (30f64).to_radians().sin()) as i32;
    fill_circle(&mut img, dot_x, dot_y, 5, light);

    img
}
