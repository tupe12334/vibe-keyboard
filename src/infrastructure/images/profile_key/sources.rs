use std::path::Path;

use image::imageops::FilterType;
use image::RgbImage;

use super::super::{
    generate_centy_image, generate_claude_image, generate_log_file_image,
    generate_numpad_entry_image, generate_terminal_image, generate_vscode_config_image,
    generate_web_image, icon_image,
};
use super::app_icon_cache::app_icon_cached;
use super::label_tile;
use crate::infrastructure::profile_store::resolve_image;

/// Image sources: `icon:<name>` (embedded icon pack), `app:<bundle id>` (installed app icon),
/// or a file path. A source that fails returns `None`, so the key falls back to its label.
/// Result is `size` by `size` pixels.
pub(in crate::infrastructure::images) fn source_image(
    image: &Path,
    root: &Path,
    size: u32,
) -> Option<RgbImage> {
    let text = image.to_string_lossy();
    if let Some(name) = text.strip_prefix("icon:") {
        return icon_image(name, size);
    }
    if let Some(bundle_id) = text.strip_prefix("app:") {
        return app_icon_cached(bundle_id, size);
    }
    load_image(&resolve_image(root, image), size)
}

fn load_image(path: &Path, size: u32) -> Option<RgbImage> {
    image::open(path).ok().map(|img| {
        img.resize_to_fill(size, size, FilterType::Triangle)
            .to_rgb8()
    })
}

pub(super) fn builtin_icon(name: &str) -> Option<RgbImage> {
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
