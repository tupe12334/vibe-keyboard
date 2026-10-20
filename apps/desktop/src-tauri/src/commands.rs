//! Profile and device commands the webview calls. Types come from the core crate.

use serde::Serialize;
use specta::Type;
use tauri::State;
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::infrastructure::profile_store;

use crate::app::App;
use crate::profile_io::{load_layout, save_layout};

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
    load_layout(&app.loaded_mtime, &profile_store::profile_path())
}

/// Write the layout. Refused while it has problems, or when the file changed on disk
/// since the editor last loaded it. Discard the edits to load the disk version first.
#[tauri::command]
#[specta::specta]
pub fn save_profile(profile: Profile, app: State<'_, App>) -> Result<(), String> {
    save_layout(&app.loaded_mtime, &profile_store::profile_path(), &profile)
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

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "commands_hold_tests.rs"]
mod hold_tests;
