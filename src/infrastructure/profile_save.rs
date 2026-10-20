//! Compare-and-write for `profile.toml`: one place that every editor saves through.
use super::profile_store::{self, save};
use crate::domain::profile::Profile;
use fs4::FileExt;
use std::collections::hash_map::DefaultHasher;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

/// Identifies the bytes of a profile file. Taken at load, compared at save. Held in memory only.
pub type Version = u64;

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

fn version_of_bytes(bytes: &[u8]) -> Version {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Version of the bytes at `path`. `None` when the file is missing.
pub fn version_of(path: &Path) -> Option<Version> {
    std::fs::read(path)
        .ok()
        .map(|bytes| version_of_bytes(&bytes))
}

/// Read the layout and the version of the exact bytes it came from. One read, so they always match.
pub fn load_with_version(path: &Path) -> Result<(Profile, Option<Version>), String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Ok((Profile::default_from_legacy(), None))
        }
        Err(e) => return Err(format!("read {}: {e}", path.display())),
    };
    let text = std::str::from_utf8(&bytes).map_err(|e| format!("read {}: {e}", path.display()))?;
    let profile = profile_store::parse(text, path)?;
    Ok((profile, Some(version_of_bytes(&bytes))))
}

/// Write `profile` only when the file still has the version `loaded`.
/// The check and the write run under an exclusive lock on `<file>.lock`, so two
/// editors cannot interleave. Threads in this process share one guard.
/// Returns the version of the file after the write.
pub fn save_if_unchanged(
    path: &Path,
    profile: &Profile,
    loaded: Option<Version>,
) -> Result<Option<Version>, SaveRefusal> {
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
    if version_of(path) != loaded {
        return Err(SaveRefusal::Changed);
    }
    save(path, profile).map_err(SaveRefusal::Failed)?;
    Ok(version_of(path))
}

#[cfg(test)]
#[path = "profile_save_tests.rs"]
mod tests;
