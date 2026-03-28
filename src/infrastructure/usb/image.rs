use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, DynamicImage};
use rusb::{Context, DeviceHandle};
use tracing::error;

use super::commands::write_cmd;
use super::{PACKET, TIMEOUT};

/// Encode image as JPEG (rotate 180°, quality 90) and send via CRT_BAT + chunks + CRT_STP.
/// `key` is 0-indexed (0–14); the device protocol uses 1-indexed key IDs.
pub fn send_button_image(handle: &DeviceHandle<Context>, key: u8, img: DynamicImage) {
    let rgb = img.rotate180().to_rgb8();
    let (w, h) = rgb.dimensions();
    let mut jpeg = Vec::new();
    if let Err(e) = JpegEncoder::new_with_quality(&mut jpeg, 90).encode(
        &rgb.into_raw(),
        w,
        h,
        ColorType::Rgb8.into(),
    ) {
        error!("JPEG encode failed: {e}");
        return;
    }

    let size = jpeg.len();
    write_cmd(
        handle,
        &[
            b'B',
            b'A',
            b'T',
            0,
            0,
            (size >> 8) as u8,
            size as u8,
            key + 1,
        ],
    );

    for chunk in jpeg.chunks(PACKET) {
        let mut pkt = [0u8; PACKET];
        pkt[..chunk.len()].copy_from_slice(chunk);
        if let Err(e) = handle.write_interrupt(super::EP_OUT, &pkt, TIMEOUT) {
            error!("image chunk write failed: {e}");
            return;
        }
    }

    write_cmd(handle, b"STP");
}
