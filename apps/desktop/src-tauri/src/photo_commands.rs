//! Command for the Google Photos field: download a shared photo into the profile's images.

use vibe_keyboard::infrastructure::google_photos::import_shared_photo;
use vibe_keyboard::infrastructure::profile_store;

/// Download the cover photo of a Google Photos share link. Gives the path to store in the
/// key's `image`, relative to the profile folder.
#[tauri::command]
#[specta::specta]
pub async fn import_google_photo(link: String) -> Result<String, String> {
    import_shared_photo(&profile_store::root_dir(), &link).map(|p| p.display().to_string())
}

#[cfg(test)]
#[path = "photo_commands_tests.rs"]
mod tests;
