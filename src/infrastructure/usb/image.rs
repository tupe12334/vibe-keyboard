use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, DynamicImage};
use tracing::error;

use super::commands::{refresh, write_cmd};
use super::{Device, CMD_PACKET, PACKET};

/// Encode image as JPEG (rotate 180°, quality 90) and send via `CRT_BAT` + chunks + `CRT_STP`.
/// `key` is 0-indexed (0–14); the device protocol uses 1-indexed key IDs.
pub fn send_button_image(handle: &Device, key: u8, img: DynamicImage) {
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
        // Image chunk output report: `0x00` (no report ID) + up to `PACKET` bytes.
        let mut pkt = [0u8; CMD_PACKET];
        pkt[1..=chunk.len()].copy_from_slice(chunk);
        if let Err(e) = handle.write(&pkt) {
            error!("image chunk write failed: {e}");
            return;
        }
    }

    refresh(handle);
}
