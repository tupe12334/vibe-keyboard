//! Key icon pack (`assets/icons/png`). The PNGs are embedded in the binary, so it works
//! from any folder and inside an app bundle.

use image::{ImageBuffer, Rgb, RgbImage};

/// Icon names usable as `image = "icon:<name>"`.
pub const ICON_NAMES: &[&str] = &[
    "back",
    "brightness-down",
    "brightness-up",
    "browser",
    "centy",
    "claude",
    "folder",
    "forward",
    "home",
    "mute",
    "numpad",
    "page-next",
    "page-prev",
    "play-pause",
    "search",
    "settings",
    "terminal",
];

/// Encoded PNG bytes for an icon name, if it exists.
pub fn icon_png(name: &str) -> Option<&'static [u8]> {
    Some(match name {
        "back" => include_bytes!("../../../assets/icons/png/back.png"),
        "brightness-down" => include_bytes!("../../../assets/icons/png/brightness-down.png"),
        "brightness-up" => include_bytes!("../../../assets/icons/png/brightness-up.png"),
        "browser" => include_bytes!("../../../assets/icons/png/browser.png"),
        "centy" => include_bytes!("../../../assets/icons/png/centy.png"),
        "claude" => include_bytes!("../../../assets/icons/png/claude.png"),
        "folder" => include_bytes!("../../../assets/icons/png/folder.png"),
        "forward" => include_bytes!("../../../assets/icons/png/forward.png"),
        "home" => include_bytes!("../../../assets/icons/png/home.png"),
        "mute" => include_bytes!("../../../assets/icons/png/mute.png"),
        "numpad" => include_bytes!("../../../assets/icons/png/numpad.png"),
        "page-next" => include_bytes!("../../../assets/icons/png/page-next.png"),
        "page-prev" => include_bytes!("../../../assets/icons/png/page-prev.png"),
        "play-pause" => include_bytes!("../../../assets/icons/png/play-pause.png"),
        "search" => include_bytes!("../../../assets/icons/png/search.png"),
        "settings" => include_bytes!("../../../assets/icons/png/settings.png"),
        "terminal" => include_bytes!("../../../assets/icons/png/terminal.png"),
        _ => return None,
    })
}

/// Decode an icon and fit it into a `size` by `size` square, centered on black.
pub fn icon_image(name: &str, size: u32) -> Option<RgbImage> {
    let decoded = image::load_from_memory(icon_png(name)?).ok()?;
    let fitted = decoded.resize(size, size, image::imageops::FilterType::Triangle);
    let mut out: RgbImage = ImageBuffer::from_pixel(size, size, Rgb([0, 0, 0]));
    let x = (size - fitted.width()) / 2;
    let y = (size - fitted.height()) / 2;
    image::imageops::overlay(&mut out, &fitted.to_rgb8(), i64::from(x), i64::from(y));
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_icon_decodes() {
        for name in ICON_NAMES {
            assert!(
                icon_image(name, 64).is_some(),
                "icon {name} does not decode"
            );
        }
        assert!(icon_png("missing").is_none());
    }
}
