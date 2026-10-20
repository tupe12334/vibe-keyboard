//! Key previews and the built-in action names, for the editor.

use std::io::Cursor;

use base64::Engine as _;
use image::RgbImage;
use vibe_keyboard::domain::profile::{Action, Profile};
use vibe_keyboard::infrastructure::images::key_image;
use vibe_keyboard::infrastructure::profile_store;
use vibe_keyboard::plugins;

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
    plugins::names().map(String::from).collect()
}

pub(crate) fn png_data_url(image: &RgbImage) -> Result<String, String> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
