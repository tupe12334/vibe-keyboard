use image::{ImageBuffer, Rgb, RgbImage};

use super::draw::draw_thick_line;

pub fn generate_sort_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);

    let bg = Rgb([15u8, 15, 15]);
    let color = Rgb([250u8, 204, 21]); // yellow

    for p in img.pixels_mut() {
        *p = bg;
    }

    // Draw three horizontal lines of decreasing length (sort icon)
    // Top line (longest)
    draw_thick_line(&mut img, 20, 30, 80, 30, color, 4);
    // Middle line
    draw_thick_line(&mut img, 28, 50, 72, 50, color, 4);
    // Bottom line (shortest)
    draw_thick_line(&mut img, 36, 70, 64, 70, color, 4);

    // Down arrow on the right side
    draw_thick_line(&mut img, 75, 35, 75, 75, color, 3);
    draw_thick_line(&mut img, 75, 75, 65, 62, color, 3);
    draw_thick_line(&mut img, 75, 75, 85, 62, color, 3);

    img
}
