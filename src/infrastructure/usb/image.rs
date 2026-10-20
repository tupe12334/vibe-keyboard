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

/// Side screen upload geometry, verified on hardware. Keys 5, 10 and 15 share the panel.
/// The upload is landscape. The panel is mounted vertically, so the portrait render is
/// rotated first (`side_upload`).
pub const SIDE_W: u16 = 480;
pub const SIDE_H: u16 = 100;

/// 32-byte side image header: `CRT\0\0` + `DRA`, size (jpeg length + 32, u32 BE), key byte
/// (always 0), then w, h, x, y as u16 BE.
pub fn side_header(jpeg_len: usize, w: u16, h: u16, x: u16, y: u16) -> [u8; 32] {
    let mut hdr = [0u8; 32];
    hdr[0..3].copy_from_slice(b"CRT");
    hdr[5..8].copy_from_slice(b"DRA");
    let size = u32::try_from(jpeg_len + 32).unwrap_or(u32::MAX);
    hdr[8..12].copy_from_slice(&size.to_be_bytes());
    hdr[13..15].copy_from_slice(&w.to_be_bytes());
    hdr[15..17].copy_from_slice(&h.to_be_bytes());
    hdr[17..19].copy_from_slice(&x.to_be_bytes());
    hdr[19..21].copy_from_slice(&y.to_be_bytes());
    hdr
}

/// Turns the portrait side render (100x480) into the landscape upload (480x100). The top
/// of the column ends at the bottom-left, so the render is rotated 90° counter-clockwise.
/// The resize keeps the upload at 480x100 whatever the input size.
pub fn side_upload(img: DynamicImage) -> DynamicImage {
    DynamicImage::ImageRgb8(
        img.rotate270()
            .resize_to_fill(
                u32::from(SIDE_W),
                u32::from(SIDE_H),
                image::imageops::FilterType::Triangle,
            )
            .to_rgb8(),
    )
}

/// Send `img` (portrait side render) to the side screen as one stream: header, then JPEG,
/// cut into output reports. Never send a taller image: it spills onto keys 5, 10 and 15.
pub fn send_side_image(handle: &Device, img: DynamicImage) {
    let rgb = side_upload(img).to_rgb8();
    let mut jpeg = Vec::new();
    if let Err(e) = JpegEncoder::new_with_quality(&mut jpeg, 90).encode(
        &rgb.into_raw(),
        u32::from(SIDE_W),
        u32::from(SIDE_H),
        ColorType::Rgb8.into(),
    ) {
        error!("side JPEG encode failed: {e}");
        return;
    }

    let mut stream = side_header(jpeg.len(), SIDE_W, SIDE_H, 0, 0).to_vec();
    stream.extend_from_slice(&jpeg);
    for chunk in stream.chunks(PACKET) {
        // Output report: `0x00` (no report ID) + up to `PACKET` bytes.
        let mut pkt = [0u8; CMD_PACKET];
        pkt[1..=chunk.len()].copy_from_slice(chunk);
        if let Err(e) = handle.write(&pkt) {
            error!("side image chunk write failed: {e}");
            return;
        }
    }

    refresh(handle);
}

#[cfg(test)]
mod tests {
    use super::{side_header, side_upload};
    use image::{DynamicImage, Rgb, RgbImage};

    #[test]
    fn side_header_bytes_match_the_verified_layout() {
        // jpeg 1000 bytes -> size field 1032 = 0x0408; w 480 = 0x01E0; h 100 = 0x0064.
        let hdr = side_header(1000, 480, 100, 0, 0);
        assert_eq!(
            &hdr[..21],
            &[
                b'C', b'R', b'T', 0, 0, b'D', b'R', b'A', 0x00, 0x00, 0x04, 0x08, 0x00, 0x01, 0xE0,
                0x00, 0x64, 0x00, 0x00, 0x00, 0x00,
            ]
        );
        assert!(hdr[21..].iter().all(|&b| b == 0));
        assert_eq!(hdr.len(), 32);
    }

    #[test]
    fn side_upload_rotates_portrait_counter_clockwise() {
        // Portrait top-left must land at landscape bottom-left; top-right at top-left.
        let mut portrait = RgbImage::new(100, 480);
        portrait.put_pixel(0, 0, Rgb([255, 0, 0]));
        portrait.put_pixel(99, 0, Rgb([0, 255, 0]));
        let out = side_upload(DynamicImage::ImageRgb8(portrait)).to_rgb8();
        assert_eq!(out.dimensions(), (480, 100));
        assert_eq!(out.get_pixel(0, 99), &Rgb([255, 0, 0]));
        assert_eq!(out.get_pixel(0, 0), &Rgb([0, 255, 0]));
    }

    #[test]
    fn side_header_carries_x_and_y() {
        let hdr = side_header(0, 480, 100, 0x0102, 0x0304);
        assert_eq!(&hdr[17..21], &[0x01, 0x02, 0x03, 0x04]);
    }
}
