use std::collections::BTreeMap;
use std::path::Path;

use image::imageops::{self, FilterType};
use image::{ImageBuffer, Rgb, RgbImage};

use super::draw::fill_circle;
use super::fit;

mod app_icon_cache;
use super::icon_image;
use super::status_tile::status_tile;
use super::text::draw_label;
use super::{
    generate_centy_image, generate_claude_image, generate_log_file_image,
    generate_numpad_entry_image, generate_terminal_image, generate_vscode_config_image,
    generate_web_image,
};
use crate::domain::actions::SessionStatus;
use crate::domain::keys::is_nav_key;
use crate::domain::profile::KEY_COUNT;
use crate::domain::profile::{inner_rect, parse_color, Action, KeyConfig, KeyInset};
use crate::infrastructure::profile_store::resolve_image;
use crate::infrastructure::usb::DECK_IMAGE_SIZE;
use app_icon_cache::app_icon_cached;
pub use app_icon_cache::app_icon_generation;

/// Render size before the inset step. Icons and labels are drawn at this size.
pub(super) const KEY_SIZE: u32 = 100;
pub(super) const FALLBACK_BG: [u8; 3] = [20, 20, 20];
pub(super) const FALLBACK_FG: [u8; 3] = [230, 230, 230];
pub(super) const BLANK: [u8; 3] = [0, 0, 0];
const NAV_BG: [u8; 3] = [40, 40, 40];

/// The bitmap the deck shows for `key`, `DECK_IMAGE_SIZE` square. The controller sends this
/// image, and the editor previews this same image. Rotation is not applied here: it is a
/// device transport detail.
pub fn key_image(key: u8, config: Option<&KeyConfig>, root: &Path, inset: &KeyInset) -> RgbImage {
    if is_nav_key(key) {
        return inset_key(&nav_image(key), inset, NAV_BG);
    }
    match config {
        Some(config) => {
            let bg = parse_color(&config.bg).unwrap_or(FALLBACK_BG);
            inset_key(&render_key(config, root), inset, bg)
        }
        None => ImageBuffer::from_pixel(DECK_IMAGE_SIZE, DECK_IMAGE_SIZE, Rgb(BLANK)),
    }
}

/// The one place key insets apply. `picture` is the full key render. Its own uniform border is
/// trimmed, then the content scales to the largest size that fits the visible rect, centered.
/// The hidden band keeps the key color `bg`.
pub(super) fn inset_key(picture: &RgbImage, inset: &KeyInset, bg: [u8; 3]) -> RgbImage {
    fit::fit_into_key(picture, inset, bg, true)
}

/// Every key 1–15 of one page, in key order. This is exactly what the controller sends.
pub fn page_images(
    keys: &BTreeMap<u8, KeyConfig>,
    root: &Path,
    inset: &KeyInset,
) -> Vec<(u8, RgbImage)> {
    (1..=KEY_COUNT)
        .map(|key| (key, key_image(key, keys.get(&key), root, inset)))
        .collect()
}

/// Picture for a configured key. Order: `image` if set, then the built-in icon for `Builtin`
/// actions, then the label on the key's colors. Relative images resolve against `root`.
fn render_key(config: &KeyConfig, root: &Path) -> RgbImage {
    if let Some(img) = config
        .image
        .as_deref()
        .and_then(|image| source_image(image, root, KEY_SIZE))
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
/// Result is `size` by `size` pixels.
pub(super) fn source_image(image: &Path, root: &Path, size: u32) -> Option<RgbImage> {
    let text = image.to_string_lossy();
    if let Some(name) = text.strip_prefix("icon:") {
        return icon_image(name, size);
    }
    if let Some(bundle_id) = text.strip_prefix("app:") {
        return app_icon_cached(bundle_id, size);
    }
    load_image(&resolve_image(root, image), size)
}

pub(super) fn label_tile(label: &str, bg: [u8; 3], fg: [u8; 3]) -> RgbImage {
    let mut img = ImageBuffer::from_pixel(KEY_SIZE, KEY_SIZE, Rgb(bg));
    draw_label(&mut img, label, Rgb(fg));
    img
}

/// Plain label tile on the default colors. Used for dynamic lists such as Claude sessions.
pub fn label_image(label: &str, inset: &KeyInset) -> RgbImage {
    inset_key(
        &label_tile(label, FALLBACK_BG, FALLBACK_FG),
        inset,
        FALLBACK_BG,
    )
}

/// A dimmed label tile for an action that does not work here, such as a text prompt in VS Code.
pub fn muted_image(label: &str, inset: &KeyInset) -> RgbImage {
    inset_key(
        &label_tile(label, [25, 25, 25], [90, 90, 90]),
        inset,
        [25, 25, 25],
    )
}

/// A Claude session tile: a status bar on top (green active, orange stale), the age in the bar's
/// right corner, and the title below, wrapped to three lines.
/// `focused` adds a white frame and a dot in the corner. The bar keeps its status colour.
pub fn session_image(
    title: &str,
    status: SessionStatus,
    age: &str,
    inset: &KeyInset,
    focused: bool,
) -> RgbImage {
    let bar = match status {
        SessionStatus::Active => [40, 180, 90],
        SessionStatus::Stale => [255, 120, 0],
    };
    let mut tile = status_tile(title, bar, age);
    if focused {
        draw_focus_marker(&mut tile);
    }
    inset_key(&tile, inset, FALLBACK_BG)
}

/// White frame of 3 px around the tile, and a white dot in the bottom-left corner.
fn draw_focus_marker(img: &mut RgbImage) {
    const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
    for (x, y, px) in img.enumerate_pixels_mut() {
        if x < 3 || y < 3 || x >= KEY_SIZE - 3 || y >= KEY_SIZE - 3 {
            *px = WHITE;
        }
    }
    fill_circle(img, 12, 87, 6, WHITE);
}

/// Keys 11, 12 and 13 always navigate. Each shows a fixed label.
fn nav_image(key: u8) -> RgbImage {
    let label = match key {
        11 => "BACK",
        12 => "OUT",
        _ => "FWD",
    };
    label_tile(label, NAV_BG, [200, 200, 200])
}

fn load_image(path: &Path, size: u32) -> Option<RgbImage> {
    image::open(path).ok().map(|img| {
        img.resize_to_fill(size, size, FilterType::Triangle)
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
        "moadim" => label_tile("MOADIM", [28, 44, 84], [230, 230, 230]),
        _ => return None,
    })
}

#[cfg(test)]
#[path = "profile_key_tests.rs"]
mod tests;
