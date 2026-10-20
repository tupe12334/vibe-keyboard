//! Lifecycle of the controller thread: one live thread at a time, and stop.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;

/// A running controller thread and the flag that stops it.
pub struct Running {
    pub(crate) shutdown: Arc<AtomicBool>,
    pub(crate) thread: JoinHandle<()>,
}

impl Running {
    fn is_alive(&self) -> bool {
        !self.thread.is_finished()
    }
}

/// Start under one lock acquisition. A live thread in `running` makes this a no-op.
/// `launch` runs while the lock is held, so it must not lock `running` itself.
pub fn start_with(
    running: &Mutex<Option<Running>>,
    launch: impl FnOnce() -> Result<Running, String>,
) -> Result<(), String> {
    let mut slot = running.lock().unwrap_or_else(PoisonError::into_inner);
    if slot.as_ref().is_some_and(Running::is_alive) {
        return Ok(());
    }
    // A thread that already ended: collect it before starting another.
    if let Some(old) = slot.take() {
        let _ = old.thread.join();
    }
    *slot = Some(launch()?);
    Ok(())
}

/// Stop the thread behind `running` and wait for it. Takes the slot, then joins outside the lock.
pub fn stop_running(running: &Mutex<Option<Running>>) {
    let taken = running
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take();
    if let Some(running) = taken {
        running.shutdown.store(true, Ordering::Relaxed);
        let _ = running.thread.join();
    }
}

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod tests;
