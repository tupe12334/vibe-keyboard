use image::{ImageBuffer, Rgb, RgbImage};

use super::draw::{bresenham, draw_thick_line, fill_circle};

pub fn generate_terminal_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);

    let canvas_bg = Rgb([15u8, 15, 15]);
    let window_body = Rgb([28u8, 28, 28]);
    let title_bar = Rgb([50u8, 50, 50]);
    let title_sep = Rgb([65u8, 65, 65]);
    let border_color = Rgb([80u8, 80, 80]);
    let dot_red = Rgb([255u8, 95, 86]);
    let dot_yellow = Rgb([255u8, 189, 46]);
    let dot_green = Rgb([39u8, 201, 63]);
    let prompt_green = Rgb([57u8, 255, 20]);

    for p in img.pixels_mut() {
        *p = canvas_bg;
    }

    for y in 12u32..=88 {
        for x in 8u32..=92 {
            img.put_pixel(x, y, window_body);
        }
    }
    for y in 12u32..=26 {
        for x in 8u32..=92 {
            img.put_pixel(x, y, title_bar);
        }
    }

    bresenham(&mut img, 8, 27, 92, 27, title_sep);
    bresenham(&mut img, 8, 12, 92, 12, border_color);
    bresenham(&mut img, 8, 88, 92, 88, border_color);
    bresenham(&mut img, 8, 12, 8, 88, border_color);
    bresenham(&mut img, 92, 12, 92, 88, border_color);

    fill_circle(&mut img, 20, 19, 3, dot_red);
    fill_circle(&mut img, 28, 19, 3, dot_yellow);
    fill_circle(&mut img, 36, 19, 3, dot_green);

    draw_thick_line(&mut img, 22, 52, 38, 42, prompt_green, 2);
    draw_thick_line(&mut img, 22, 52, 38, 62, prompt_green, 2);
    draw_thick_line(&mut img, 44, 63, 58, 63, prompt_green, 2);

    img
}
