//! Commands for the app-icon field: running apps for the picker, and the bundle id rule.

use serde::Serialize;
use specta::Type;
use vibe_keyboard::domain::profile::is_bundle_id;
use vibe_keyboard::infrastructure::running_apps::running_apps;

#[derive(Serialize, Type)]
pub struct RunningApp {
    pub name: String,
    pub bundle_id: String,
}

/// Apps running now, for the app-icon picker. Empty if the list cannot be read.
#[tauri::command]
#[specta::specta]
pub async fn list_running_apps() -> Vec<RunningApp> {
    running_apps()
        .into_iter()
        .map(|(name, bundle_id)| RunningApp { name, bundle_id })
        .collect()
}

/// Whether `text` is a bundle id, by the core rule the layout uses for `open_app`.
#[tauri::command]
#[specta::specta]
pub fn check_bundle_id(text: String) -> bool {
    is_bundle_id(&text)
}

#[cfg(test)]
#[path = "app_commands_tests.rs"]
mod tests;
