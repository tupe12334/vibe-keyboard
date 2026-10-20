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
