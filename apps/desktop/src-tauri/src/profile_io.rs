//! Load and save of the layout for the editor. The commands call these; the tests run them on
//! a real file, so the command path is covered without the webview.
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use serde::Serialize;
use specta::Type;
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::domain::profile_rules;
use vibe_keyboard::infrastructure::profile_save::{
    load_with_version, save_if_unchanged, version_of, SaveRefusal, Version,
};

/// Why a save did not write. The webview reads `kind`.
#[derive(Debug, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SaveError {
    /// The file changed on disk since the editor loaded it. Nothing was written.
    Conflict,
    /// Nothing was written. The message says why.
    Refused { message: String },
}

impl From<SaveRefusal> for SaveError {
    fn from(refusal: SaveRefusal) -> Self {
        match refusal {
            SaveRefusal::Changed => Self::Conflict,
            SaveRefusal::Failed(message) => Self::Refused { message },
        }
    }
}

/// Read the layout at `path` and remember the version of the bytes it came from in `loaded`.
pub fn load_layout(loaded: &Mutex<Option<Version>>, path: &Path) -> Result<Profile, String> {
    let (profile, version) = load_with_version(path)?;
    *loaded.lock().unwrap_or_else(PoisonError::into_inner) = version;
    Ok(profile)
}

/// Write `profile` to `path` if the file still has the version remembered in `loaded`. With
/// `force`, write over whatever is on disk. Records the version it wrote. Refuses a layout
/// with problems before touching the file.
pub fn save_layout(
    loaded: &Mutex<Option<Version>>,
    path: &Path,
    profile: &Profile,
    force: bool,
) -> Result<(), SaveError> {
    let problems = profile_rules::save_problems(profile);
    if !problems.is_empty() {
        return Err(SaveError::Refused {
            message: format!("{} problem(s): {}", problems.len(), problems.join("; ")),
        });
    }
    let base = if force {
        version_of(path)
    } else {
        *loaded.lock().unwrap_or_else(PoisonError::into_inner)
    };
    let written = save_if_unchanged(path, profile, base)?;
    *loaded.lock().unwrap_or_else(PoisonError::into_inner) = written;
    Ok(())
}

#[cfg(test)]
#[path = "profile_io_tests.rs"]
mod tests;
