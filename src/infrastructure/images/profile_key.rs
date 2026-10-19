use std::collections::BTreeMap;
use std::path::Path;

use image::imageops::FilterType;
use image::{ImageBuffer, Rgb, RgbImage};

use super::icon_image;
use super::text::draw_label;
use super::{
    generate_centy_image, generate_claude_image, generate_log_file_image,
    generate_numpad_entry_image, generate_terminal_image, generate_vscode_config_image,
    generate_web_image,
};
use crate::domain::keys::is_nav_key;
use crate::domain::profile::KEY_COUNT;
use crate::domain::profile::{parse_color, Action, KeyConfig};
use crate::infrastructure::app_icon::app_icon;
use crate::infrastructure::profile_store::resolve_image;

const KEY_SIZE: u32 = 100;
const FALLBACK_BG: [u8; 3] = [20, 20, 20];
const FALLBACK_FG: [u8; 3] = [230, 230, 230];
const BLANK: [u8; 3] = [0, 0, 0];

/// The bitmap the deck shows for `key`. The controller sends this image, and the editor
/// previews this same image. Rotation is not applied here: it is a device transport detail.
pub fn key_image(key: u8, config: Option<&KeyConfig>, root: &Path) -> RgbImage {
    if is_nav_key(key) {
        return nav_image(key);
    }
    match config {
        Some(config) => render_key(config, root),
        None => ImageBuffer::from_pixel(KEY_SIZE, KEY_SIZE, Rgb(BLANK)),
    }
}

/// Every key 1–15 of one page, in key order. This is exactly what the controller sends.
pub fn page_images(keys: &BTreeMap<u8, KeyConfig>, root: &Path) -> Vec<(u8, RgbImage)> {
    (1..=KEY_COUNT)
        .map(|key| (key, key_image(key, keys.get(&key), root)))
        .collect()
}

/// Picture for a configured key. Order: `image` if set, then the built-in icon for `Builtin`
/// actions, then the label on the key's colors. Relative images resolve against `root`.
fn render_key(config: &KeyConfig, root: &Path) -> RgbImage {
    if let Some(img) = config
        .image
        .as_deref()
        .and_then(|image| source_image(image, root))
    {
        return img;
    }
    if let Action::Builtin { name } = &config.action {
        if let Some(icon) = builtin_icon(name) {
            return icon;
        }
    }
    let bg = parse_color(&config.bg).unwrap_or(FALLBACK_BG);
    let fg = parse_color(&config.fg).unwrap_or(FALLBACK_FG);
    label_tile(&config.label, bg, fg)
}

/// Image sources: `icon:<name>` (embedded icon pack), `app:<bundle id>` (installed app icon),
/// or a file path. A source that fails returns `None`, so the key falls back to its label.
fn source_image(image: &Path, root: &Path) -> Option<RgbImage> {
    let text = image.to_string_lossy();
    if let Some(name) = text.strip_prefix("icon:") {
        return icon_image(name, KEY_SIZE);
    }
    if let Some(bundle_id) = text.strip_prefix("app:") {
        return app_icon(bundle_id, KEY_SIZE).map(|img| {
            img.resize_to_fill(KEY_SIZE, KEY_SIZE, FilterType::Triangle)
                .to_rgb8()
        });
    }
    load_image(&resolve_image(root, image))
}

fn label_tile(label: &str, bg: [u8; 3], fg: [u8; 3]) -> RgbImage {
    let mut img = ImageBuffer::from_pixel(KEY_SIZE, KEY_SIZE, Rgb(bg));
    draw_label(&mut img, label, Rgb(fg));
    img
}

/// Keys 11, 12 and 13 always navigate. Each shows a fixed label.
fn nav_image(key: u8) -> RgbImage {
    let label = match key {
        11 => "BACK",
        12 => "OUT",
        _ => "FWD",
    };
    label_tile(label, [40, 40, 40], [200, 200, 200])
}

fn load_image(path: &Path) -> Option<RgbImage> {
    image::open(path).ok().map(|img| {
        img.resize_to_fill(KEY_SIZE, KEY_SIZE, FilterType::Triangle)
            .to_rgb8()
    })
}

fn builtin_icon(name: &str) -> Option<RgbImage> {
    Some(match name {
        "centy_projects" => generate_centy_image(),
        "terminal" => generate_terminal_image(),
        "claude" => generate_claude_image(),
        "centy_web" => generate_web_image(),
        "numpad" => generate_numpad_entry_image(),
        "log_file" => generate_log_file_image(),
        "vscode_config" => generate_vscode_config_image(),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::Profile;
    use std::path::PathBuf;

    #[test]
    fn every_key_gives_a_square_bitmap() {
        let profile = Profile::default_from_legacy();
        let root = PathBuf::from("/nonexistent");
        for key in 1..=15u8 {
            let config = profile.key_at(0, key);
            let img = key_image(key, config, &root);
            assert_eq!(img.dimensions(), (KEY_SIZE, KEY_SIZE), "key {key}");
        }
    }

    #[test]
    fn page_images_lists_every_key_once_in_order() {
        let profile = Profile::default_from_legacy();
        let root = PathBuf::from("/nonexistent");
        let keys = &profile.pages[0].keys;
        let images = page_images(keys, &root);
        let numbers: Vec<u8> = images.iter().map(|(key, _)| *key).collect();
        assert_eq!(numbers, (1..=15).collect::<Vec<u8>>());
        for (key, img) in images {
            assert_eq!(img, key_image(key, keys.get(&key), &root), "key {key}");
        }
    }

    #[test]
    fn nav_keys_do_not_depend_on_the_profile() {
        let root = PathBuf::from("/nonexistent");
        let config = KeyConfig {
            label: "x".into(),
            ..KeyConfig::default()
        };
        assert_eq!(
            key_image(11, Some(&config), &root),
            key_image(11, None, &root)
        );
        assert_ne!(key_image(12, None, &root), key_image(13, None, &root));
    }

    #[test]
    fn empty_key_is_black_and_icon_sources_fall_back() {
        let root = PathBuf::from("/nonexistent");
        let blank = key_image(5, None, &root);
        assert!(blank.pixels().all(|p| p.0 == BLANK));
        let icon = KeyConfig {
            image: Some(PathBuf::from("icon:back")),
            label: "Hi".into(),
            ..KeyConfig::default()
        };
        assert_eq!(
            key_image(5, Some(&icon), &root),
            icon_image("back", KEY_SIZE).unwrap_or_default()
        );
        // An app that is not installed falls back to the label.
        let app = KeyConfig {
            image: Some(PathBuf::from("app:com.example.NotInstalled")),
            label: "Hi".into(),
            ..KeyConfig::default()
        };
        assert_eq!(
            key_image(5, Some(&app), &root),
            label_tile("Hi", FALLBACK_BG, FALLBACK_FG)
        );
    }
}
