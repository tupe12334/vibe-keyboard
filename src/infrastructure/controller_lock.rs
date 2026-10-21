//! Single-instance guard for the controller.
//!
//! hidapi opens the deck non-exclusively, so two controller processes both
//! receive key presses and both act on them. The first process to take this
//! lock owns the deck. The OS releases the lock when that process exits or crashes.

use fs4::{FileExt, TryLockError};
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

const LOCK_FILE: &str = "vibe-keyboard-controller.lock";

/// Exclusive lock on the lock file. Keep it alive for the whole process.
#[derive(Debug)]
pub struct ControllerLock {
    _file: File,
}

#[derive(Debug)]
pub enum AcquireError {
    /// Another controller holds the lock.
    Held,
    Io(io::Error),
}

impl From<TryLockError> for AcquireError {
    fn from(e: TryLockError) -> Self {
        let e = io::Error::from(e);
        if e.kind() == io::ErrorKind::WouldBlock {
            Self::Held
        } else {
            Self::Io(e)
        }
    }
}

/// Lock file in the user runtime dir. macOS has no runtime dir, so it uses the per-user temp dir.
pub fn default_lock_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(LOCK_FILE)
}

/// Take an exclusive, non-blocking lock on `path`. Creates the file if missing.
pub fn acquire(path: &Path) -> Result<ControllerLock, AcquireError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(AcquireError::Io)?;
    match FileExt::try_lock(&file) {
        Ok(()) => Ok(ControllerLock { _file: file }),
        Err(e) => Err(AcquireError::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::expect_used, reason = "test: setup failures should panic")]
    fn second_acquire_reports_held_until_first_is_dropped() {
        let dir = std::env::temp_dir().join(format!(
            "vibe-keyboard-lock-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        ));
        std::fs::create_dir_all(&dir).expect("create test dir");
        let path = dir.join(LOCK_FILE);

        let first = acquire(&path).expect("first acquire succeeds");
        assert!(matches!(acquire(&path), Err(AcquireError::Held)));

        drop(first); // Fork/exec race: a concurrent fork holds our fd until exec, so retry.
        let second = (0..100).find_map(|_| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            acquire(&path).ok()
        });
        assert!(second.is_some(), "lock is free after the holder drops it");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn lock_io_error_is_not_reported_as_held() {
        let err = AcquireError::from(TryLockError::Error(io::Error::other("disk")));
        assert!(matches!(err, AcquireError::Io(_)));
    }

    #[test]
    fn default_path_is_the_lock_file() {
        assert!(default_lock_path().ends_with(LOCK_FILE));
    }
}
