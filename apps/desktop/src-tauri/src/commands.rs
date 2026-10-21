//! Profile and device commands the webview calls. Types come from the core crate.

use serde::Serialize;
use specta::Type;
use std::sync::atomic::Ordering;
use tauri::State;
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::domain::profile_rules;
use vibe_keyboard::infrastructure::profile_store;

use crate::app::App;
use crate::editor_log;
use crate::profile_io::{load_layout, save_layout, SaveError};

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
    app.editor_loaded.store(true, Ordering::SeqCst);
    load_layout(&app.loaded_version, &profile_store::profile_path())
}

/// Write the layout. Refused while it has problems. Refused with `Conflict` when the file changed
/// on disk since the editor last loaded it, unless `force` overwrites the disk version.
#[tauri::command]
#[specta::specta]
pub fn save_profile(profile: Profile, force: bool, app: State<'_, App>) -> Result<(), SaveError> {
    let path = profile_store::profile_path();
    let result = save_layout(&app.loaded_version, &path, &profile, force);
    match &result {
        Ok(()) => tracing::info!(
            "{}",
            editor_log::saved_line(&profile, &path.display().to_string())
        ),
        Err(err) => {
            let line = match err {
                // Problem text can carry action payloads, so only the count is logged.
                SaveError::Refused { .. } => match profile_rules::save_problems(&profile).len() {
                    0 => editor_log::save_failed_line("write error, details not logged"),
                    count => editor_log::save_refused_line(count),
                },
                SaveError::Conflict => editor_log::save_failed_line("file changed on disk"),
            };
            tracing::warn!("{}", line);
        }
    }
    result
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

/// The root crate's build line: `vibe-keyboard <version> (<sha>, built <date>)`. The footer shows it.
#[tauri::command]
#[specta::specta]
pub fn build_info() -> String {
    vibe_keyboard::build_info::startup_line()
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "commands_hold_tests.rs"]
mod hold_tests;
