mod images;
mod navigation;
mod state;
mod tui;

use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, DynamicImage};
use images::generate_terminal_image;
use navigation::Navigator;
use rusb::{Context, DeviceHandle, UsbContext as _};
use state::DeviceState;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const VID: u16 = 0x0300;
const PID: u16 = 0x3010;
const EP_OUT: u8 = 0x03;
const EP_IN: u8 = 0x82;
const PACKET: usize = 512;
const TIMEOUT: Duration = Duration::from_millis(1000);

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

// ── USB transport ─────────────────────────────────────────────────────────────

fn make_cmd(body: &[u8]) -> [u8; PACKET] {
    let mut pkt = [0u8; PACKET];
    pkt[..5].copy_from_slice(b"CRT\0\0");
    pkt[5..5 + body.len()].copy_from_slice(body);
    pkt
}

fn write_cmd(handle: &DeviceHandle<Context>, body: &[u8]) {
    handle
        .write_interrupt(EP_OUT, &make_cmd(body), TIMEOUT)
        .expect("write_cmd failed");
}

fn device_init(handle: &DeviceHandle<Context>) {
    write_cmd(handle, b"DIS");
    write_cmd(handle, &[b'L', b'I', b'G', 0, 0, 0, 0]);
}

fn set_brightness(handle: &DeviceHandle<Context>, percent: u8) {
    write_cmd(handle, &[b'L', b'I', b'G', 0, 0, percent]);
}

fn clear_all(handle: &DeviceHandle<Context>) {
    write_cmd(handle, &[b'C', b'L', b'E', 0, 0, 0, 0xff]);
}

/// Encode image as JPEG (rotate 180°, quality 90) and send via CRT_BAT + chunks + CRT_STP.
/// `key` is 0-indexed (0–14); the device protocol uses 1-indexed key IDs.
fn send_button_image(handle: &DeviceHandle<Context>, key: u8, img: DynamicImage) {
    let rgb = img.rotate180().to_rgb8();
    let (w, h) = rgb.dimensions();
    let mut jpeg = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg, 90)
        .encode(&rgb.into_raw(), w, h, ColorType::Rgb8.into())
        .expect("JPEG encode failed");

    let size = jpeg.len();
    write_cmd(handle, &[b'B', b'A', b'T', 0, 0, (size >> 8) as u8, size as u8, key + 1]);

    for chunk in jpeg.chunks(PACKET) {
        let mut pkt = [0u8; PACKET];
        pkt[..chunk.len()].copy_from_slice(chunk);
        handle
            .write_interrupt(EP_OUT, &pkt, TIMEOUT)
            .expect("image chunk write failed");
    }

    write_cmd(handle, b"STP");
}

fn keep_alive(handle: &DeviceHandle<Context>) {
    write_cmd(handle, b"CONNECT");
}

fn read_event(handle: &DeviceHandle<Context>, timeout: Duration) -> Option<Vec<u8>> {
    let mut buf = vec![0u8; PACKET];
    match handle.read_interrupt(EP_IN, &mut buf, timeout) {
        Ok(_) => Some(buf),
        Err(rusb::Error::Timeout) => None,
        Err(e) => { eprintln!("[reader] {e}"); None }
    }
}

// ── Page / key logic ──────────────────────────────────────────────────────────

fn activate_page(
    page: usize,
    handle: &DeviceHandle<Context>,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    state.lock().unwrap().current_page = page;
    {
        let mut ds = dev_state.lock().unwrap();
        ds.current_page = page;
        ds.save();
    }
    clear_all(handle);
    match page {
        0 => {
            send_button_image(handle, 2, DynamicImage::ImageRgb8(generate_terminal_image()));
            state.lock().unwrap().push_log("page 0: terminal".into());
        }
        1 => {
            state.lock().unwrap().push_log("page 1: (empty)".into());
        }
        _ => {}
    }
}

fn handle_key_event(
    buf: &[u8],
    handle: &DeviceHandle<Context>,
    nav: &mut Navigator,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let raw_id = buf[9];
    let state_byte = buf[10];
    if raw_id == 0x00 || raw_id == 0xFF { return; }
    let state_str = match state_byte { 1 => "pressed", 2 => "released", _ => return };
    let key = match raw_to_logical(raw_id) {
        Some(k) => k,
        None => return,
    };
    {
        let mut s = state.lock().unwrap();
        s.pressed_key = if state_byte == 1 { Some(key) } else { None };
        if state_byte == 1 { s.push_log(format!("key {:2}  {state_str}", key)); }
    }
    if state_byte != 1 { return; }
    match key {
        11 => {
            let page = nav.back();
            state.lock().unwrap().push_log(format!("← back → page {}", page + 1));
            activate_page(page, handle, state, dev_state);
        }
        12 => {
            let page = nav.forward();
            state.lock().unwrap().push_log(format!("→ forward → page {}", page + 1));
            activate_page(page, handle, state, dev_state);
        }
        _ => {
            if nav.current() == 0 && key == 2 {
                state.lock().unwrap().push_log("opening Terminal".into());
                open_terminal();
            }
        }
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let ctx = Context::new().expect("rusb context");
    let handle = ctx
        .open_device_with_vid_pid(VID, PID)
        .expect("Device not found — is it plugged in? Try: sudo cargo run");
    handle
        .claim_interface(0)
        .expect("Failed to claim USB interface 0 — try: sudo cargo run");
    println!("[init] interface 0 claimed");

    let dev_state_val = DeviceState::load();

    device_init(&handle);
    clear_all(&handle);
    set_brightness(&handle, dev_state_val.brightness);
    println!("[init] device ready");

    let app_state = Arc::new(Mutex::new(tui::AppState::new(2)));
    let shutdown  = Arc::new(AtomicBool::new(false));
    let dev_state = Arc::new(Mutex::new(dev_state_val));

    {
        let s = Arc::clone(&app_state);
        let q = Arc::clone(&shutdown);
        std::thread::spawn(move || tui::run(s, q));
    }

    let initial_page = dev_state.lock().unwrap().current_page;
    let mut nav = Navigator::new(2);
    nav.go(initial_page);
    activate_page(nav.current(), &handle, &app_state, &dev_state);
    println!("[init] listening — press keys (11=back, 12=forward)");

    let mut last_heartbeat = Instant::now();

    loop {
        if shutdown.load(Ordering::Relaxed) { break; }

        if let Some(data) = read_event(&handle, Duration::from_millis(500)) {
            if data.len() >= 11 {
                handle_key_event(&data, &handle, &mut nav, &app_state, &dev_state);
            }
        }

        if last_heartbeat.elapsed() >= Duration::from_secs(8) {
            keep_alive(&handle);
            last_heartbeat = Instant::now();
        }
    }
}
