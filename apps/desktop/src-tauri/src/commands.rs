//! Commands the webview calls. Types come from the core crate, so bindings stay in sync.

use std::io::Cursor;

use base64::Engine as _;
use image::RgbImage;
use serde::Serialize;
use specta::Type;
use tauri::State;
use vibe_keyboard::domain::profile::{Profile, BUILTIN_NAMES};
use vibe_keyboard::infrastructure::images::key_image;
use vibe_keyboard::infrastructure::profile_store;

use crate::app::App;

#[derive(Serialize, Type)]
pub struct DeviceStatus {
    pub connected: bool,
    pub running: bool,
}

/// The saved layout. Missing file gives the built-in layout.
#[tauri::command]
#[specta::specta]
pub fn get_profile(app: State<'_, App>) -> Result<Profile, String> {
    let path = profile_store::profile_path();
    let profile = profile_store::read(&path)?.unwrap_or_else(Profile::default_from_legacy);
    app.record_profile_mtime();
    Ok(profile)
}

/// Write the layout. Refused while it has problems, or when the file changed on disk
/// since the editor last loaded it. Discard the edits to load the disk version first.
#[tauri::command]
#[specta::specta]
pub fn save_profile(profile: Profile, app: State<'_, App>) -> Result<(), String> {
    let problems = profile.problems();
    if !problems.is_empty() {
        return Err(format!(
            "{} problem(s): {}",
            problems.len(),
            problems.join("; ")
        ));
    }
    if profile.pages.is_empty() {
        return Err("the layout needs at least one page".into());
    }
    let path = profile_store::profile_path();
    let loaded = *app
        .loaded_mtime
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if profile_store::modified_at(&path) != loaded {
        return Err("profile.toml changed on disk. Discard your edits to load it.".into());
    }
    profile_store::save(&path, &profile)?;
    app.record_profile_mtime();
    Ok(())
}

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

#[tauri::command]
#[specta::specta]
pub fn list_builtins() -> Vec<String> {
    BUILTIN_NAMES.iter().map(ToString::to_string).collect()
}

#[tauri::command]
#[specta::specta]
pub fn device_status(app: State<'_, App>) -> DeviceStatus {
    DeviceStatus {
        connected: app.is_device_connected(),
        running: app.is_running(),
    }
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
