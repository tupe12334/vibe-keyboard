//! Compare-and-write for `profile.toml`: one place that every editor saves through.
use super::profile_store::{modified_at, save};
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

/// Write `profile` only when the file still has the modification time `loaded`.
/// The check and the write run under an exclusive lock on `<file>.lock`, so two
/// editors cannot interleave. Threads in this process share one guard.
pub fn save_if_unchanged(
    path: &Path,
    profile: &Profile,
    loaded: Option<SystemTime>,
) -> Result<(), SaveRefusal> {
    static IN_PROCESS: Mutex<()> = Mutex::new(());
    let _guard = IN_PROCESS.lock().unwrap_or_else(PoisonError::into_inner);
    let lock_path = path.with_extension("toml.lock");
    let lock = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|e| SaveRefusal::Failed(format!("lock {}: {e}", lock_path.display())))?;
    FileExt::lock(&lock)
        .map_err(|e| SaveRefusal::Failed(format!("lock {}: {e}", lock_path.display())))?;
    if modified_at(path) != loaded {
        return Err(SaveRefusal::Changed);
    }
    save(path, profile).map_err(SaveRefusal::Failed)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn temp_profile(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("vibe-keyboard-save-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create dir");
        dir.join("profile.toml")
    }

    #[test]
    fn writes_when_the_file_is_as_loaded_and_refuses_when_it_changed() {
        let path = temp_profile("guard");
        std::fs::write(&path, "").expect("seed file");
        let loaded = modified_at(&path);
        let profile = Profile::default_from_legacy();
        assert_eq!(save_if_unchanged(&path, &profile, loaded), Ok(()));

        let stale = Some(SystemTime::now() - Duration::from_secs(3600));
        assert_eq!(
            save_if_unchanged(&path, &profile, stale),
            Err(SaveRefusal::Changed)
        );
    }
}
