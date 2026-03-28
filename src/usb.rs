use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, DynamicImage};
use rusb::{Context, DeviceHandle};
use std::time::Duration;

pub const VID: u16 = 0x0300;
pub const PID: u16 = 0x3010;
pub const EP_OUT: u8 = 0x03;
pub const EP_IN: u8 = 0x82;
pub const PACKET: usize = 512;
pub const TIMEOUT: Duration = Duration::from_millis(1000);

fn make_cmd(body: &[u8]) -> [u8; PACKET] {
    let mut pkt = [0u8; PACKET];
    pkt[..5].copy_from_slice(b"CRT\0\0");
    pkt[5..5 + body.len()].copy_from_slice(body);
    pkt
}

pub fn write_cmd(handle: &DeviceHandle<Context>, body: &[u8]) {
    handle
        .write_interrupt(EP_OUT, &make_cmd(body), TIMEOUT)
        .expect("write_cmd failed");
}

pub fn device_init(handle: &DeviceHandle<Context>) {
    write_cmd(handle, b"DIS");
    write_cmd(handle, &[b'L', b'I', b'G', 0, 0, 0, 0]);
}

pub fn set_brightness(handle: &DeviceHandle<Context>, percent: u8) {
    write_cmd(handle, &[b'L', b'I', b'G', 0, 0, percent]);
}

pub fn clear_all(handle: &DeviceHandle<Context>) {
    write_cmd(handle, &[b'C', b'L', b'E', 0, 0, 0, 0xff]);
}

/// Encode image as JPEG (rotate 180°, quality 90) and send via CRT_BAT + chunks + CRT_STP.
/// `key` is 0-indexed (0–14); the device protocol uses 1-indexed key IDs.
pub fn send_button_image(handle: &DeviceHandle<Context>, key: u8, img: DynamicImage) {
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

pub fn keep_alive(handle: &DeviceHandle<Context>) {
    write_cmd(handle, b"CONNECT");
}

pub fn read_event(handle: &DeviceHandle<Context>, timeout: Duration) -> Option<Vec<u8>> {
    let mut buf = vec![0u8; PACKET];
    match handle.read_interrupt(EP_IN, &mut buf, timeout) {
        Ok(_) => Some(buf),
        Err(rusb::Error::Timeout) => None,
        Err(e) => { eprintln!("[reader] {e}"); None }
    }
}
