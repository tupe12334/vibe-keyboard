//! One disconnect and reconnect through the real device loop, on a fake deck.
#![allow(
    clippy::expect_used,
    reason = "test module: setup failures should panic"
)]
use super::event::ControllerEvent;
use super::options::RunOptions;
use super::reconnect_fake::{handle, Counts};
use super::reconnect_port::FakeDeck;
use super::run::run_with;
use crate::presentation::tui::AppState;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, PoisonError};

/// Writes `open_device` sends: `device_init`. The loop's stop sends one more (`clear_all`).
const INIT_WRITES: usize = 6;
const STOP_WRITES: usize = 1;

#[test]
fn a_lost_deck_is_reconnected_redrawn_and_read_again() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let first = Arc::new(Mutex::new(Counts::default()));
    let second = Arc::new(Mutex::new(Counts::default()));
    let deck = FakeDeck::new(vec![
        handle(true, &first, &shutdown),
        handle(false, &second, &shutdown),
    ]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);

    let result = run_with(
        RunOptions {
            headless: true,
            shutdown: Arc::clone(&shutdown),
            state: Arc::new(Mutex::new(AppState::new(1))),
            on_event: Box::new(move |e: ControllerEvent| {
                sink.lock().unwrap_or_else(PoisonError::into_inner).push(e);
            }),
        },
        &deck,
    );

    assert!(result.is_ok());
    let events = seen.lock().unwrap_or_else(PoisonError::into_inner).clone();
    assert_eq!(
        events,
        vec![
            ControllerEvent::DeviceReady,
            ControllerEvent::DeviceLost,
            ControllerEvent::DeviceRestored,
            ControllerEvent::Stopped,
        ]
    );
    assert_eq!(deck.opens.get(), 2, "reconnect opened a new handle");
    assert_eq!(deck.clears.get(), 2, "reconnect cleared the stuck button");
    assert_eq!(
        first.lock().unwrap_or_else(PoisonError::into_inner).reads,
        1
    );
    let after = second.lock().unwrap_or_else(PoisonError::into_inner);
    assert!(after.reads >= 1, "the loop read the new handle");
    assert!(
        after.writes > INIT_WRITES + STOP_WRITES,
        "the screen was redrawn on the new handle"
    );
}
