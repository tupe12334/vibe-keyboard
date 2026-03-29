use image::{ImageBuffer, Rgb, RgbImage};

use super::draw::draw_thick_line;

pub fn generate_search_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);

    let bg = Rgb([15u8, 15, 15]);
    let color = Rgb([250u8, 204, 21]); // yellow

    for p in img.pixels_mut() {
        *p = bg;
    }

    // Magnifying glass circle
    for angle_deg in 0..360u32 {
        let angle = (angle_deg as f64) * std::f64::consts::PI / 180.0;
        for r in 22u32..=26 {
            let x = (40.0 + (r as f64) * angle.cos()).round() as i32;
            let y = (38.0 + (r as f64) * angle.sin()).round() as i32;
            if (0..100).contains(&x) && (0..100).contains(&y) {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }

    // Magnifying glass handle
    draw_thick_line(&mut img, 60, 58, 80, 78, color, 5);

    img
}
