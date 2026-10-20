//! Controller: owns the deck, runs the device loop, and reports what happens.
//! `listen` and the desktop app both call `run`.
mod ctx;
mod event;
mod input;
mod keys;
mod reload;
mod run;
mod setup;
mod side;
mod stop;
mod tick;

use crate::infrastructure::controller_lock::{self, AcquireError, ControllerLock};
use std::path::Path;

pub use event::ControllerEvent;
pub use run::{run, RunOptions};
pub use setup::{load_profile, load_profile_at};

/// Take the single-controller lock. Keep the guard alive for the whole run.
pub fn claim() -> Result<ControllerLock, AcquireError> {
    claim_at(&controller_lock::default_lock_path())
}

/// `claim` on a given lock file.
pub fn claim_at(path: &Path) -> Result<ControllerLock, AcquireError> {
    controller_lock::acquire(path)
}

/// Text for a refused claim. `listen` and the desktop app show the same text.
pub fn held_message(path: &Path) -> String {
    format!(
        "another vibe-keyboard controller is already running (lock {})",
        path.display()
    )
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn second_claim_is_refused_until_the_first_is_dropped() {
        let dir = std::env::temp_dir().join(format!("vibe-keyboard-claim-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create dir");
        let path = dir.join("claim.lock");

        let first = claim_at(&path).expect("first claim");
        assert!(matches!(claim_at(&path), Err(AcquireError::Held)));
        assert!(held_message(&path).contains("already running"));
        drop(first);
        assert!(claim_at(&path).is_ok());
    }
}
