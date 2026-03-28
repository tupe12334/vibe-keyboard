mod navigation;
mod tui;

use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use mirajazz::device::{list_devices, Device, DeviceQuery};
use mirajazz::types::{DeviceInput, ImageFormat, ImageMirroring, ImageMode, ImageRotation};
use navigation::Navigator;
use rusb::UsbContext as _;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
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

async fn activate_page(page: usize, device: &Device, state: &Arc<Mutex<tui::AppState>>) {
    {
        let mut s = state.lock().unwrap();
        s.current_page = page;
    }
    device.clear_all_button_images().await.ok();
    match page {
        0 => {
            let term_img = generate_terminal_image();
            device
                .set_button_image(2, IMAGE_FORMAT, DynamicImage::ImageRgb8(term_img))
                .await
                .ok();
            device.flush().await.ok();
            state.lock().unwrap().push_log("page 0: terminal".into());
        }
        1 => {
            state.lock().unwrap().push_log("page 1: (empty)".into());
        }
        _ => {}
    }
}

async fn handle_key_event(
    buf: &[u8],
    device: &Device,
    nav: &mut Navigator,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let raw_id    = buf[9];
    let state_byte = buf[10];
    if raw_id == 0x00 { return; }
    if raw_id == 0xFF { return; }
    let state_str = match state_byte { 1 => "pressed", 2 => "released", _ => return };
    let key = match raw_to_logical(raw_id) {
        Some(k) => k,
        None => return,
    };

    {
        let mut s = state.lock().unwrap();
        s.pressed_key = if state_byte == 1 { Some(key) } else { None };
        if state_byte == 1 {
            s.push_log(format!("key {:2}  {state_str}", key));
        }
    }

    if state_byte != 1 { return; }

    match key {
        11 => {
            let page = nav.back();
            state.lock().unwrap().push_log(format!("← back → page {}", page + 1));
            activate_page(page, device, state).await;
        }
        12 => {
            let page = nav.forward();
            state.lock().unwrap().push_log(format!("→ forward → page {}", page + 1));
            activate_page(page, device, state).await;
        }
        _ => match nav.current() {
            0 => {
                if key == 2 {
                    state.lock().unwrap().push_log("opening Terminal".into());
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

fn generate_terminal_image() -> RgbImage {
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

/// Evicts the AppleUserHIDDrivers DriverKit dext by briefly seizing USB interface 0
/// (USBInterfaceOpenSeize), then releasing it immediately. This leaves the interface free
/// for the in-kernel IOUSBHIDDriver to match and register an IOHIDDevice service, which is
/// what async-hid (mirajazz) needs. Holding the claim would block IOUSBHIDDriver.
fn evict_dext() {
    let ctx = rusb::Context::new().expect("rusb context");
    let handle = ctx
        .open_device_with_vid_pid(VID, PID)
        .expect("USB device not found — is it plugged in?");
    handle.claim_interface(0).expect(
        "Failed to claim USB interface 0. Try running with sudo.",
    );
    handle.release_interface(0).ok();
    // handle (and ctx) dropped here — interface is now free for IOUSBHIDDriver
    println!("[init] AppleUserHIDDrivers dext evicted");
}

#[tokio::main]
async fn main() {
    evict_dext();

    // Give macOS time to re-run IOKit driver matching and register the IOUSBHIDDriver
    // IOHIDDevice service before we enumerate.
    tokio::time::sleep(Duration::from_millis(1000)).await;

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

    let app_state = Arc::new(Mutex::new(tui::AppState::new(2)));
    let shutdown  = Arc::new(AtomicBool::new(false));

    // Spawn TUI on a blocking thread
    {
        let s = Arc::clone(&app_state);
        let q = Arc::clone(&shutdown);
        std::thread::spawn(move || tui::run(s, q));
    }

    let mut nav = Navigator::new(2);
    activate_page(nav.current(), &device, &app_state).await;

    let reader = device.get_reader(|_, _| Ok(DeviceInput::NoData));
    let mut last_heartbeat = Instant::now();

    loop {
        if shutdown.load(Ordering::Relaxed) { break; }

        match reader.raw_read_data_with_timeout(512, Duration::from_millis(500)).await {
            Ok(Some(data)) if data.len() >= 11 => {
                handle_key_event(&data, &device, &mut nav, &app_state).await;
            }
            Ok(_) => {}
            Err(e) => { app_state.lock().unwrap().push_log(format!("[reader] {e}")); }
        }

        if last_heartbeat.elapsed() >= Duration::from_secs(8) {
            device.keep_alive().await.ok();
            last_heartbeat = Instant::now();
        }
    }
}
