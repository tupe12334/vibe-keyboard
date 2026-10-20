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

pub use event::ControllerEvent;
pub use run::{run, RunOptions};
pub use setup::load_profile;

use crate::infrastructure::controller_lock::{self, AcquireError, ControllerLock};

/// Take the single-controller lock. Keep the guard alive for the whole run.
pub fn claim() -> Result<ControllerLock, AcquireError> {
    // A panic must not leave a push-to-talk key down.
    crate::application::install_panic_release();
    controller_lock::acquire(&controller_lock::default_lock_path())
}
