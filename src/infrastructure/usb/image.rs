use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, DynamicImage};
use tracing::error;

use super::commands::{refresh, write_cmd};
use super::{Device, CMD_PACKET, PACKET};

mod side_format;
use crate::domain::keys::logical_to_raw;
use side_format::{side_header, side_upload, SIDE_H, SIDE_W};

/// Key image edge in pixels, as verified on the AKP153E.
pub const DECK_IMAGE_SIZE: u32 = 85;

/// Encode `img` as JPEG and send it to logical key `key` (1–15, same numbering as the
/// editor grid and input events). The device wants the physical id in `BAT`, and it shows
/// images turned 90° clockwise, so the image is pre-rotated counter-clockwise.
pub fn send_button_image(handle: &Device, key: u8, img: DynamicImage) {
    let Some(raw) = logical_to_raw(key) else {
        error!("send_button_image: logical key {key} is out of range 1..=15");
        return;
    };
    let rgb = img
        .resize_exact(
            DECK_IMAGE_SIZE,
            DECK_IMAGE_SIZE,
            image::imageops::FilterType::Triangle,
        )
        .rotate270()
        .to_rgb8();
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
        &[b'B', b'A', b'T', 0, 0, (size >> 8) as u8, size as u8, raw],
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

/// Send `img` (portrait side render) to the side screen as one stream: header, then JPEG,
/// cut into output reports. Never send a taller image: it spills onto keys 5, 10 and 15.
/// Returns `false` when the frame did not reach the device, so the caller can retry it.
pub fn send_side_image(handle: &Device, img: DynamicImage) -> bool {
    let rgb = side_upload(img).to_rgb8();
    let mut jpeg = Vec::new();
    if let Err(e) = JpegEncoder::new_with_quality(&mut jpeg, 90).encode(
        &rgb.into_raw(),
        u32::from(SIDE_W),
        u32::from(SIDE_H),
        ColorType::Rgb8.into(),
    ) {
        error!("side JPEG encode failed: {e}");
        return false;
    }

    let mut stream = side_header(jpeg.len(), SIDE_W, SIDE_H, 0, 0).to_vec();
    stream.extend_from_slice(&jpeg);
    for chunk in stream.chunks(PACKET) {
        // Output report: `0x00` (no report ID) + up to `PACKET` bytes.
        let mut pkt = [0u8; CMD_PACKET];
        pkt[1..=chunk.len()].copy_from_slice(chunk);
        if let Err(e) = handle.write(&pkt) {
            error!("side image chunk write failed: {e}");
            return false;
        }
    }

    refresh(handle);
    true
}

#[cfg(test)]
#[path = "image_tests.rs"]
mod tests;
