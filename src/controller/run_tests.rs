//! `run` when shutdown is already requested: it returns Ok, reports `Stopped`, and never reports `DeviceReady`.
use super::event::ControllerEvent;
use super::run::{run, RunOptions};
use crate::presentation::tui::AppState;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[test]
fn shutdown_while_waiting_for_device_returns_cleanly() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    let result = run(RunOptions {
        headless: true,
        shutdown: Arc::new(AtomicBool::new(true)),
        state: Arc::new(Mutex::new(AppState::new(0))),
        on_event: Box::new(move |e: ControllerEvent| {
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(e);
        }),
    });
    assert!(result.is_ok());
    let events = seen
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(*events, vec![ControllerEvent::Stopped]);
}
