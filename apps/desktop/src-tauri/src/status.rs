//! Read-only status of the controller, for the tray and the editor.
use std::sync::atomic::Ordering;
use std::sync::PoisonError;

use crate::app::App;

impl App {
    pub fn is_running(&self) -> bool {
        self.running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .is_some_and(|running| !running.thread.is_finished())
    }

    pub fn is_device_connected(&self) -> bool {
        self.device_connected.load(Ordering::Relaxed)
    }

    pub fn last_error(&self) -> Option<String> {
        self.error
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub(crate) fn set_error(&self, error: Option<String>) {
        *self.error.lock().unwrap_or_else(PoisonError::into_inner) = error;
    }
}
