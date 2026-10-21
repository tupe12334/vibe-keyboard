//! What a caller passes to `run`: the stop flag, the UI state, and the event sink.
use super::event::ControllerEvent;
use crate::presentation::tui::AppState;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

/// Called on the controller thread for each event.
pub type EventSink = Box<dyn Fn(ControllerEvent) + Send>;

pub struct RunOptions {
    /// Skip the ratatui TUI. No terminal is needed.
    pub headless: bool,
    /// Set this to stop the loop. Signal handlers set it too.
    pub shutdown: Arc<AtomicBool>,
    /// UI state. Call `load_profile` on it only after `claim()` succeeds.
    pub state: Arc<Mutex<AppState>>,
    pub on_event: EventSink,
}
