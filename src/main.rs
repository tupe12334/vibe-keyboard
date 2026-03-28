mod navigation;

use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use mirajazz::device::{list_devices, Device, DeviceQuery};
use mirajazz::types::{DeviceInput, ImageFormat, ImageMirroring, ImageMode, ImageRotation};
use navigation::Navigator;
use std::f64::consts::PI;
use std::process::Command;
use std::time::{Duration, Instant};

const VID: u16 = 0x0300;
const PID: u16 = 0x3010;

// usage_page 65440 (0xFFE0) and usage_id 1 are standard for Ajazz/Mirabox vendor HID interfaces
const QUERY: DeviceQuery = DeviceQuery::new(65440, 1, VID, PID);

const IMAGE_FORMAT: ImageFormat = ImageFormat {
    mode: ImageMode::JPEG,
    size: (100, 100),
    rotation: ImageRotation::Rot180,
    mirror: ImageMirroring::None,
};

fn raw_to_logical(raw: u8) -> Option<u8> {
    match raw {
        0x0D => Some(1),  0x0A => Some(2),  0x07 => Some(3),  0x04 => Some(4),  0x01 => Some(5),
        0x0E => Some(6),  0x0B => Some(7),  0x08 => Some(8),  0x05 => Some(9),  0x02 => Some(10),
        0x0F => Some(11), 0x0C => Some(12), 0x09 => Some(13), 0x06 => Some(14), 0x03 => Some(15),
        _ => None,
    }
}

fn open_terminal() {
    Command::new("open")
        .arg("-a")
        .arg("Terminal")
        .spawn()
        .unwrap_or_else(|e| { eprintln!("Failed to open Terminal: {e}"); std::process::exit(1) });
}

async fn activate_page(page: usize, device: &Device) {
    device.clear_all_button_images().await.ok();
    match page {
        0 => {
            let clock_img = generate_clock_image();
            device
                .set_button_image(2, IMAGE_FORMAT, DynamicImage::ImageRgb8(clock_img))
                .await
                .ok();
            device.flush().await.ok();
            println!("[nav] page 0: clock");
        }
        1 => {
            println!("[nav] page 1: (empty)");
        }
        _ => {}
    }
}

async fn handle_key_event(buf: &[u8], device: &Device, nav: &mut Navigator) {
    let raw_id = buf[9];
    let state  = buf[10];
    if raw_id == 0x00 { return; }
    if raw_id == 0xFF { println!("[ack]"); return; }
    let state_str = match state { 1 => "pressed", 2 => "released", s => { println!("state={s:#04x}"); return; } };
    let key = match raw_to_logical(raw_id) {
        Some(k) => k,
        None => { println!("unknown raw_id={raw_id:#04x} state={state:#04x}"); return; }
    };
    println!("key {key:2}  {state_str}");
    if state != 1 { return; } // act only on press

    match key {
        11 => {
            let page = nav.back();
            println!("← back → page {page}");
            activate_page(page, device).await;
        }
        12 => {
            let page = nav.forward();
            println!("→ forward → page {page}");
            activate_page(page, device).await;
        }
        _ => match nav.current() {
            0 => {
                if key == 2 {
                    println!("→ opening Terminal");
                    open_terminal();
                }
            }
            _ => {}
        },
    }
}

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
        if e2 < dx  { err += dx;  y += sy; }
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

fn generate_clock_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);
    let (cx, cy) = (50.0f64, 50.0f64);

    for p in img.pixels_mut() { *p = Rgb([18u8, 18u8, 40u8]); }

    for y in 0..100u32 {
        for x in 0..100u32 {
            let d = (((x as f64 - cx).powi(2) + (y as f64 - cy).powi(2)) as f64).sqrt();
            if d <= 44.0 { img.put_pixel(x, y, Rgb([230, 230, 255])); }
            if d > 41.0 && d <= 44.0 { img.put_pixel(x, y, Rgb([80, 80, 130])); }
        }
    }

    for h in 0u32..12 {
        let a = h as f64 * PI / 6.0 - PI / 2.0;
        let (r1, r2) = if h % 3 == 0 { (34.0, 41.0) } else { (38.0, 41.0) };
        bresenham(&mut img,
            (cx + r1 * a.cos()) as i32, (cy + r1 * a.sin()) as i32,
            (cx + r2 * a.cos()) as i32, (cy + r2 * a.sin()) as i32,
            Rgb([40, 40, 80]));
    }

    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let mins  = (secs / 60 % 60) as f64;
    let hours = (secs / 3600 % 12) as f64 + mins / 60.0;

    let ha = hours * PI / 6.0 - PI / 2.0;
    draw_thick_line(&mut img, cx as i32, cy as i32,
        (cx + 22.0 * ha.cos()) as i32, (cy + 22.0 * ha.sin()) as i32,
        Rgb([20, 20, 60]), 3);

    let ma = mins * PI / 30.0 - PI / 2.0;
    draw_thick_line(&mut img, cx as i32, cy as i32,
        (cx + 33.0 * ma.cos()) as i32, (cy + 33.0 * ma.sin()) as i32,
        Rgb([20, 20, 60]), 2);

    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            if dx * dx + dy * dy <= 5 {
                let px = (cx as i32 + dx).max(0) as u32;
                let py = (cy as i32 + dy).max(0) as u32;
                if px < 100 && py < 100 { img.put_pixel(px, py, Rgb([20, 20, 60])); }
            }
        }
    }

    img
}

#[tokio::main]
async fn main() {
    let devices = list_devices(&[QUERY]).await.expect("HID enumerate failed");
    if devices.is_empty() {
        eprintln!("Device {:04x}:{:04x} not found.", VID, PID);
        std::process::exit(1);
    }

    let dev_info = devices.into_iter().next().unwrap();
    let device = Device::connect(&dev_info, 1, 15, 3).await.expect("connect failed");
    println!("[init] firmware: {:?}", device.firmware_version);

    device.clear_all_button_images().await.expect("clear failed");
    device.set_brightness(25).await.expect("brightness failed");

    let mut nav = Navigator::new(2);
    activate_page(nav.current(), &device).await;

    let reader = device.get_reader(|_, _| Ok(DeviceInput::NoData));

    println!("[init] listening — press keys (11=back, 12=forward)\n");
    let mut last_heartbeat = Instant::now();

    loop {
        match reader.raw_read_data_with_timeout(512, Duration::from_millis(500)).await {
            Ok(Some(data)) if data.len() >= 11 => handle_key_event(&data, &device, &mut nav).await,
            Ok(_) => {}
            Err(e) => eprintln!("[reader] {e}"),
        }

        if last_heartbeat.elapsed() >= Duration::from_secs(8) {
            device.keep_alive().await.ok();
            last_heartbeat = Instant::now();
        }
    }
}
