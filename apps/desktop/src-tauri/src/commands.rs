//! Profile and device commands the webview calls. Types come from the core crate.

use std::sync::PoisonError;

use serde::Serialize;
use specta::Type;
use tauri::State;
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::domain::profile_rules;
use vibe_keyboard::infrastructure::profile_save::save_if_unchanged;
use vibe_keyboard::infrastructure::profile_store;

use crate::app::App;

#[derive(Serialize, Type)]
pub struct DeviceStatus {
    pub connected: bool,
    pub running: bool,
    /// Why the controller refused to start or stopped with an error. Absent when fine.
    pub error: Option<String>,
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
    let problems = profile_rules::save_problems(&profile);
    if !problems.is_empty() {
        return Err(format!(
            "{} problem(s): {}",
            problems.len(),
            problems.join("; ")
        ));
    }
    let path = profile_store::profile_path();
    let loaded = *app
        .loaded_mtime
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    match save_if_unchanged(&path, &profile, loaded) {
        Ok(()) => {
            app.record_profile_mtime();
            Ok(())
        }
        Err(refusal) => Err(refusal.to_string()),
    }
}

#[tauri::command]
#[specta::specta]
pub fn device_status(app: State<'_, App>) -> DeviceStatus {
    DeviceStatus {
        connected: app.is_device_connected(),
        running: app.is_running(),
        error: app.last_error(),
    }
}
