//! Key and side previews and the built-in action names, for the editor.

use std::io::Cursor;

use base64::Engine as _;
use image::RgbImage;
use vibe_keyboard::domain::profile::{Action, Profile, BUILTIN_NAMES};
use vibe_keyboard::infrastructure::images::{key_image, side_image};
use vibe_keyboard::infrastructure::profile_store;
use vibe_keyboard::infrastructure::system_metrics::{Disk, Metrics};

/// PNG data URL of one key, drawn by the same `key_image` the deck uses.
/// `profile` is the layout to preview, which may be unsaved.
#[tauri::command]
#[specta::specta]
pub fn key_preview(profile: Profile, page: u32, key: u8) -> Result<String, String> {
    let image = key_image(
        key,
        profile.key_at(page as usize, key),
        &profile_store::root_dir(),
        &profile.display.key_inset,
    );
    png_data_url(&image)
}

/// PNG data URL of the side screen of `page`, drawn by the same `side_image` the deck uses.
/// The values are a fixed sample, so the preview reads the same on every machine.
#[tauri::command]
#[specta::specta]
pub fn side_preview(profile: Profile, page: u32) -> Result<String, String> {
    let side = profile
        .pages
        .get(page as usize)
        .and_then(|p| p.side.as_ref());
    let image = side_image(
        side,
        &profile_store::root_dir(),
        &sample_metrics(),
        &profile.display.side_inset,
    );
    png_data_url(&image)
}

fn sample_metrics() -> Metrics {
    Metrics {
        cpu_percent: 62,
        disk: Some(Disk {
            free: 412_000_000_000,
            total: 994_000_000_000,
        }),
        temperature: Some(58.0),
    }
}

/// Why an action cannot run, from the core rule. `None` when it can. `page_count` is the
/// number of pages in the layout being edited.
#[tauri::command]
#[specta::specta]
pub fn action_problem(action: Action, page_count: u32) -> Option<String> {
    action.problem(page_count as usize)
}

#[tauri::command]
#[specta::specta]
pub fn list_builtins() -> Vec<String> {
    BUILTIN_NAMES.iter().map(ToString::to_string).collect()
}

fn png_data_url(image: &RgbImage) -> Result<String, String> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_preview_is_a_png_for_a_page_with_and_without_a_side_screen() {
        let mut profile = Profile::default_from_legacy();
        let bare = side_preview(profile.clone(), 0);
        assert!(bare.is_ok_and(|url| url.starts_with("data:image/png;base64,")));
        profile.set_side(
            0,
            Some(vibe_keyboard::domain::profile::SideConfig::default()),
        );
        assert!(side_preview(profile.clone(), 0).is_ok());
        assert!(
            side_preview(profile, 9).is_ok(),
            "a missing page shows a black column"
        );
    }
}
