use image::{ImageBuffer, Rgb, RgbImage};

fn bresenham(img: &mut RgbImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb<u8>) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1i32 } else { -1 };
    let sy = if y0 < y1 { 1i32 } else { -1 };
    let mut err = dx - dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        if x >= 0 && x < 100 && y >= 0 && y < 100 {
            img.put_pixel(x as u32, y as u32, color);
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 > -dy { err -= dy; x += sx; }
        if e2 < dx  { err += dx; y += sy; }
    }
}

fn draw_thick_line(img: &mut RgbImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb<u8>, thickness: i32) {
    let half = thickness / 2;
    let dx = (x1 - x0) as f64;
    let dy = (y1 - y0) as f64;
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 { return; }
    for t in -half..=half {
        let ox = (-dy / len * t as f64).round() as i32;
        let oy = ( dx / len * t as f64).round() as i32;
        bresenham(img, x0 + ox, y0 + oy, x1 + ox, y1 + oy, color);
    }
}

fn fill_circle(img: &mut RgbImage, cx: i32, cy: i32, r: i32, color: Rgb<u8>) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < 100 && py >= 0 && py < 100 {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
}

pub fn generate_terminal_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);

    let canvas_bg    = Rgb([15u8,  15,  15]);
    let window_body  = Rgb([28u8,  28,  28]);
    let title_bar    = Rgb([50u8,  50,  50]);
    let title_sep    = Rgb([65u8,  65,  65]);
    let border_color = Rgb([80u8,  80,  80]);
    let dot_red      = Rgb([255u8, 95,  86]);
    let dot_yellow   = Rgb([255u8, 189, 46]);
    let dot_green    = Rgb([39u8,  201, 63]);
    let prompt_green = Rgb([57u8,  255, 20]);

    for p in img.pixels_mut() { *p = canvas_bg; }

    for y in 12u32..=88 { for x in 8u32..=92 { img.put_pixel(x, y, window_body); } }
    for y in 12u32..=26 { for x in 8u32..=92 { img.put_pixel(x, y, title_bar); } }

    bresenham(&mut img, 8, 27, 92, 27, title_sep);
    bresenham(&mut img, 8,  12, 92, 12, border_color);
    bresenham(&mut img, 8,  88, 92, 88, border_color);
    bresenham(&mut img, 8,  12,  8, 88, border_color);
    bresenham(&mut img, 92, 12, 92, 88, border_color);

    fill_circle(&mut img, 20, 19, 3, dot_red);
    fill_circle(&mut img, 28, 19, 3, dot_yellow);
    fill_circle(&mut img, 36, 19, 3, dot_green);

    draw_thick_line(&mut img, 22, 52, 38, 42, prompt_green, 2);
    draw_thick_line(&mut img, 22, 52, 38, 62, prompt_green, 2);
    draw_thick_line(&mut img, 44, 63, 58, 63, prompt_green, 2);

    img
}
