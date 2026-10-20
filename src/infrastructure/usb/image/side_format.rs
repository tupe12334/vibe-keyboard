use image::DynamicImage;

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
pub fn side_upload(img: &DynamicImage) -> DynamicImage {
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
