use image::{ImageBuffer, Rgb, RgbImage};

/// Draw a filled rectangle on the image.
fn fill_rect(img: &mut RgbImage, x0: u32, y0: u32, x1: u32, y1: u32, color: Rgb<u8>) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            if x < 100 && y < 100 {
                img.put_pixel(x, y, color);
            }
        }
    }
}

mod buttons;
mod digits;

pub use buttons::{generate_numpad_backspace_image, generate_numpad_clear_image};
pub use digits::generate_numpad_digit_image;
#[cfg(test)]
use digits::segments;

/// Generate a 100×100 "numpad entry" icon for the main-page button.
pub fn generate_numpad_entry_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);
    let bg = Rgb([15u8, 15, 15]);
    let fg = Rgb([100u8, 200, 100]);

    for p in img.pixels_mut() {
        *p = bg;
    }

    // Draw a simple grid of 9 small squares representing a numpad
    let offsets: [(u32, u32); 9] = [
        (15, 15),
        (42, 15),
        (69, 15),
        (15, 42),
        (42, 42),
        (69, 42),
        (15, 69),
        (42, 69),
        (69, 69),
    ];
    for (ox, oy) in offsets {
        fill_rect(&mut img, ox, oy, ox + 16, oy + 16, fg);
    }

    img
}

#[cfg(test)]
#[path = "numpad_tests.rs"]
mod tests;
