//! Load and save of the layout for the editor. The commands call these; the tests run them on
//! a real file, so the command path is covered without the webview.
use std::path::Path;
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::domain::profile_rules;
use vibe_keyboard::infrastructure::profile_save::{load_with_mtime, save_if_unchanged};

/// Read the layout at `path` and remember the file time it was read at in `loaded`.
pub fn load_layout(loaded: &Mutex<Option<SystemTime>>, path: &Path) -> Result<Profile, String> {
    let (profile, mtime) = load_with_mtime(path)?;
    *loaded.lock().unwrap_or_else(PoisonError::into_inner) = mtime;
    Ok(profile)
}

/// Write `profile` to `path` if the file still has the time remembered in `loaded`. Records the
/// time it wrote. Refuses a layout with problems before touching the file.
pub fn save_layout(
    loaded: &Mutex<Option<SystemTime>>,
    path: &Path,
    profile: &Profile,
) -> Result<(), String> {
    let problems = profile_rules::save_problems(profile);
    if !problems.is_empty() {
        return Err(format!(
            "{} problem(s): {}",
            problems.len(),
            problems.join("; ")
        ));
    }
    let base = *loaded.lock().unwrap_or_else(PoisonError::into_inner);
    let written = save_if_unchanged(path, profile, base).map_err(|r| r.to_string())?;
    *loaded.lock().unwrap_or_else(PoisonError::into_inner) = written;
    Ok(())
}

#[cfg(test)]
#[path = "profile_io_tests.rs"]
mod tests;
