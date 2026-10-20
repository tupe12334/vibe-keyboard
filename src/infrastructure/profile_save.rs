//! Compare-and-write for `profile.toml`: one place that every editor saves through.
use super::profile_store::{self, modified_at, save};
use crate::domain::profile::Profile;
use fs4::FileExt;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

/// Why a save did not write.
#[derive(Debug, PartialEq, Eq)]
pub enum SaveRefusal {
    /// The file changed on disk after the editor loaded it.
    Changed,
    Failed(String),
}

impl std::fmt::Display for SaveRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Changed => {
                f.write_str("profile.toml changed on disk. Discard your edits to load it.")
            }
            Self::Failed(e) => f.write_str(e),
        }
    }
}

/// Read the layout and the modification time it was read at. The time is taken first: an edit
/// made during the read leaves an older time, so the next save is refused, not overwritten.
pub fn load_with_mtime(path: &Path) -> Result<(Profile, Option<SystemTime>), String> {
    let mtime = modified_at(path);
    let profile = profile_store::read(path)?.unwrap_or_else(Profile::default_from_legacy);
    Ok((profile, mtime))
}

/// Write `profile` only when the file still has the modification time `loaded`.
/// The check and the write run under an exclusive lock on `<file>.lock`, so two
/// editors cannot interleave. Threads in this process share one guard.
/// Returns the modification time of the file after the write.
pub fn save_if_unchanged(
    path: &Path,
    profile: &Profile,
    loaded: Option<SystemTime>,
) -> Result<Option<SystemTime>, SaveRefusal> {
    static IN_PROCESS: Mutex<()> = Mutex::new(());
    let _guard = IN_PROCESS.lock().unwrap_or_else(PoisonError::into_inner);
    let lock_path = path.with_extension("toml.lock");
    // Held until the function returns: the write happens under this lock.
    let _lock = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .and_then(|file| FileExt::lock(&file).map(|()| file))
        .map_err(|e| SaveRefusal::Failed(format!("lock {}: {e}", lock_path.display())))?;
    if modified_at(path) != loaded {
        return Err(SaveRefusal::Changed);
    }
    save(path, profile).map_err(SaveRefusal::Failed)?;
    Ok(modified_at(path))
}

#[cfg(test)]
#[path = "profile_save_tests.rs"]
mod tests;
