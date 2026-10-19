use std::path::Path;

use image::{imageops, ImageBuffer, Rgb, RgbImage};

use super::profile_key::source_image;
use super::text::draw_label;
use crate::domain::profile::{parse_color, SideConfig};

/// Side screen size in pixels. Never send a taller image: it spills onto keys 5, 10 and 15.
pub const SIDE_WIDTH: u32 = 480;
pub const SIDE_HEIGHT: u32 = 100;
/// Visible part of the side screen, as x offset and width. Keep content inside it.
pub const SAFE_X: u32 = 30;
pub const SAFE_WIDTH: u32 = 410;

const FALLBACK_BG: [u8; 3] = [20, 20, 20];
const FALLBACK_FG: [u8; 3] = [230, 230, 230];
const BLANK: [u8; 3] = [0, 0, 0];

/// The bitmap the side screen shows. The controller sends this image, and the editor
/// previews this same image. `None` gives a black strip.
pub fn side_image(side: Option<&SideConfig>, root: &Path) -> RgbImage {
    let Some(side) = side else {
        return ImageBuffer::from_pixel(SIDE_WIDTH, SIDE_HEIGHT, Rgb(BLANK));
    };
    let bg = parse_color(&side.bg).unwrap_or(FALLBACK_BG);
    let fg = parse_color(&side.fg).unwrap_or(FALLBACK_FG);
    let mut canvas = ImageBuffer::from_pixel(SIDE_WIDTH, SIDE_HEIGHT, Rgb(bg));
    // Content is drawn in the safe box, then placed at its x offset.
    let mut safe = ImageBuffer::from_pixel(SAFE_WIDTH, SIDE_HEIGHT, Rgb(bg));
    let picture = side
        .image
        .as_deref()
        .and_then(|image| source_image(image, root, SIDE_HEIGHT));
    match picture {
        Some(img) => {
            let left = (SAFE_WIDTH - SIDE_HEIGHT) / 2;
            imageops::replace(&mut safe, &img, i64::from(left), 0);
        }
        None => draw_label(&mut safe, &side.label, Rgb(fg)),
    }
    imageops::replace(&mut canvas, &safe, i64::from(SAFE_X), 0);
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn side_image_is_480_by_100() {
        let root = PathBuf::from("/nonexistent");
        assert_eq!(side_image(None, &root).dimensions(), (480, 100));
        let side = SideConfig {
            label: "Build ok".into(),
            ..SideConfig::default()
        };
        assert_eq!(side_image(Some(&side), &root).dimensions(), (480, 100));
    }

    #[test]
    fn label_stays_inside_the_safe_area() {
        let root = PathBuf::from("/nonexistent");
        let side = SideConfig {
            label: "LONG LABEL THAT WRAPS ACROSS LINES".into(),
            bg: "#000000".into(),
            fg: "#FFFFFF".into(),
            image: None,
        };
        let img = side_image(Some(&side), &root);
        let inside = |x: u32| (SAFE_X..SAFE_X + SAFE_WIDTH).contains(&x);
        for (x, _, pixel) in img.enumerate_pixels() {
            if pixel.0 == [255, 255, 255] {
                assert!(
                    inside(x),
                    "foreground pixel at x={x} is outside the safe area"
                );
            }
        }
    }
}
