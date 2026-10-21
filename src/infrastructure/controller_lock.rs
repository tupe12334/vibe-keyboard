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
#[path = "controller_lock_tests.rs"]
mod tests;
