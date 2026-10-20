//! Runs a controller body on its own thread and reports how it ended, panic or not.
use std::thread::JoinHandle;

/// Reason given when the body panicked.
pub const PANIC_REASON: &str = "controller stopped unexpectedly. See app.log.";

/// Calls `on_exit` when the thread ends. A panic unwinding through the thread counts as unexpected.
struct ExitGuard<F: FnOnce(Option<String>)> {
    on_exit: Option<F>,
    failure: Option<String>,
}

impl<F: FnOnce(Option<String>)> Drop for ExitGuard<F> {
    fn drop(&mut self) {
        let reason = if std::thread::panicking() {
            Some(PANIC_REASON.to_string())
        } else {
            self.failure.take()
        };
        if let Some(on_exit) = self.on_exit.take() {
            on_exit(reason);
        }
    }
}

/// Run `body` on a new thread. `body` returns the failure reason, or None for a clean exit.
/// `on_exit` runs on the thread after `body`. It must not wait for the main thread, which may
/// be joining this thread.
pub fn spawn_controller(
    body: impl FnOnce() -> Option<String> + Send + 'static,
    on_exit: impl FnOnce(Option<String>) + Send + 'static,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut guard = ExitGuard {
            on_exit: Some(on_exit),
            failure: None,
        };
        guard.failure = body();
    })
}

#[cfg(test)]
#[path = "supervise_tests.rs"]
mod tests;
