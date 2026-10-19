use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use image::imageops::FilterType;
use image::{ImageBuffer, Rgb, RgbImage};

use super::icon_image;
use super::text::{draw_label, draw_title};
use super::{
    generate_centy_image, generate_claude_image, generate_log_file_image,
    generate_numpad_entry_image, generate_terminal_image, generate_vscode_config_image,
    generate_web_image,
};
use crate::domain::actions::SessionStatus;
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

/// Calibration tile: the logical key number in big white digits on black.
pub fn calibration_image(key: u8) -> RgbImage {
    label_tile(&key.to_string(), [0, 0, 0], [255, 255, 255])
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
        return app_icon_cached(bundle_id);
    }
    load_image(&resolve_image(root, image))
}

/// Installed-app icons, looked up off-thread. Results are cached, misses included.
#[derive(Default)]
struct AppIconCache {
    /// Finished lookups. `None` is a miss (no such app, or no icon).
    ready: HashMap<String, Option<RgbImage>>,
    /// Lookups still running on a worker thread.
    pending: HashSet<String>,
}

static APP_ICONS: OnceLock<Mutex<AppIconCache>> = OnceLock::new();
/// Bumped whenever a lookup finishes, so callers know to re-render.
static APP_ICON_GENERATION: AtomicU64 = AtomicU64::new(0);

fn app_cache() -> std::sync::MutexGuard<'static, AppIconCache> {
    APP_ICONS
        .get_or_init(|| Mutex::new(AppIconCache::default()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Counter that changes each time an app icon lookup finishes.
pub fn app_icon_generation() -> u64 {
    APP_ICON_GENERATION.load(Ordering::Relaxed)
}

/// The icon for `bundle_id` if it is already known. Otherwise start a background lookup and
/// return `None`, so the key shows its label until the lookup finishes.
fn app_icon_cached(bundle_id: &str) -> Option<RgbImage> {
    let mut cache = app_cache();
    if let Some(known) = cache.ready.get(bundle_id) {
        return known.clone();
    }
    if cache.pending.insert(bundle_id.to_string()) {
        let id = bundle_id.to_string();
        std::thread::spawn(move || {
            let icon = app_icon(&id, KEY_SIZE).map(|img| {
                img.resize_to_fill(KEY_SIZE, KEY_SIZE, FilterType::Triangle)
                    .to_rgb8()
            });
            let mut cache = app_cache();
            cache.pending.remove(&id);
            cache.ready.insert(id, icon);
            APP_ICON_GENERATION.fetch_add(1, Ordering::Relaxed);
        });
    }
    None
}

fn label_tile(label: &str, bg: [u8; 3], fg: [u8; 3]) -> RgbImage {
    let mut img = ImageBuffer::from_pixel(KEY_SIZE, KEY_SIZE, Rgb(bg));
    draw_label(&mut img, label, Rgb(fg));
    img
}

/// Plain label tile on the default colors. Used for dynamic lists such as Claude sessions.
pub fn label_image(label: &str) -> RgbImage {
    label_tile(label, FALLBACK_BG, FALLBACK_FG)
}

/// A Claude session tile: a status bar on top (amber waiting, green working, grey unknown) and
/// the title below, wrapped to three lines.
pub fn session_image(title: &str, status: SessionStatus) -> RgbImage {
    const BAR_H: u32 = 12;
    let bar = match status {
        SessionStatus::Waiting => [230, 160, 0],
        SessionStatus::Working => [40, 180, 90],
        SessionStatus::Unknown => [90, 90, 90],
    };
    let mut body = ImageBuffer::from_pixel(KEY_SIZE, KEY_SIZE - BAR_H, Rgb(FALLBACK_BG));
    draw_title(&mut body, &title.to_uppercase(), Rgb(FALLBACK_FG));
    let mut img = ImageBuffer::from_pixel(KEY_SIZE, KEY_SIZE, Rgb(bar));
    image::imageops::replace(&mut img, &body, 0, i64::from(BAR_H));
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
