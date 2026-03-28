use image::{Rgb, RgbImage};

pub fn bresenham(img: &mut RgbImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb<u8>) {
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

pub fn draw_thick_line(img: &mut RgbImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb<u8>, thickness: i32) {
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

pub fn fill_circle(img: &mut RgbImage, cx: i32, cy: i32, r: i32, color: Rgb<u8>) {
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
