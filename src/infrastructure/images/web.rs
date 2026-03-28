use image::{ImageBuffer, Rgb, RgbImage};

use super::draw::draw_thick_line;

pub fn generate_web_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);

    let bg = Rgb([15u8, 15, 15]);
    let globe = Rgb([56u8, 189, 248]); // sky blue
    let dark = Rgb([14u8, 116, 144]);

    for p in img.pixels_mut() {
        *p = bg;
    }

    // Globe circle outline
    for angle_deg in 0..360u32 {
        let angle = (angle_deg as f64) * std::f64::consts::PI / 180.0;
        for r in 34u32..=38 {
            let x = (50.0 + (r as f64) * angle.cos()).round() as i32;
            let y = (50.0 + (r as f64) * angle.sin()).round() as i32;
            if x >= 0 && x < 100 && y >= 0 && y < 100 {
                img.put_pixel(x as u32, y as u32, globe);
            }
        }
    }

    // Horizontal latitude lines
    for &y in &[38u32, 50, 62] {
        let half_w = (((35.0f64).powi(2) - ((y as f64) - 50.0).powi(2))
            .max(0.0)
            .sqrt()) as u32;
        if half_w > 0 {
            let x_start = 50u32.saturating_sub(half_w);
            let x_end = (50 + half_w).min(99);
            for x in x_start..=x_end {
                img.put_pixel(x, y, dark);
            }
        }
    }

    // Vertical central meridian
    for y in 15u32..=85 {
        img.put_pixel(50, y, dark);
    }

    // Ellipse for the middle vertical arc (simplified)
    draw_thick_line(&mut img, 50, 15, 35, 50, dark, 1);
    draw_thick_line(&mut img, 35, 50, 50, 85, dark, 1);
    draw_thick_line(&mut img, 50, 15, 65, 50, dark, 1);
    draw_thick_line(&mut img, 65, 50, 50, 85, dark, 1);

    img
}
