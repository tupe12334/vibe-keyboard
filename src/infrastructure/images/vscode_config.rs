use image::{ImageBuffer, Rgb, RgbImage};

use super::draw::{draw_thick_line, fill_circle};

pub fn generate_vscode_config_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);

    let bg         = Rgb([20u8,  20,  20]);
    let blue_dark  = Rgb([0u8,   75, 135]);
    let blue_light = Rgb([0u8,  120, 212]);
    let white      = Rgb([255u8, 255, 255]);
    let gray       = Rgb([180u8, 180, 180]);

    for p in img.pixels_mut() { *p = bg; }

    // VS Code icon background square with rounded feel (filled rectangle)
    for y in 10u32..=85 { for x in 15u32..=85 { img.put_pixel(x, y, blue_dark); } }

    // VS Code chevron shape: two thick ">" marks forming the logo
    // Left part of the VS Code logo — a stylized "<" on the left
    draw_thick_line(&mut img, 30, 35, 50, 48, blue_light, 4);
    draw_thick_line(&mut img, 30, 63, 50, 50, blue_light, 4);

    // Right part — "> <" shape
    draw_thick_line(&mut img, 50, 48, 70, 35, white, 4);
    draw_thick_line(&mut img, 50, 50, 70, 63, white, 4);

    // Small gear overlay in bottom-right to indicate config
    let gear_cx = 72i32;
    let gear_cy = 72i32;

    // Gear body
    fill_circle(&mut img, gear_cx, gear_cy, 9, bg);
    fill_circle(&mut img, gear_cx, gear_cy, 7, gray);
    fill_circle(&mut img, gear_cx, gear_cy, 4, bg);

    // Gear teeth (8 teeth)
    for i in 0..8 {
        let angle = (i as f64) * std::f64::consts::PI / 4.0;
        let tx = (gear_cx as f64 + angle.cos() * 10.0).round() as i32;
        let ty = (gear_cy as f64 + angle.sin() * 10.0).round() as i32;
        fill_circle(&mut img, tx, ty, 2, gray);
    }

    img
}
