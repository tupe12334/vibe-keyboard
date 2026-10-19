use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, DynamicImage};
use tracing::error;

use super::commands::{refresh, write_cmd};
use super::{Device, CMD_PACKET, PACKET};
use crate::domain::keys::logical_to_raw;

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
