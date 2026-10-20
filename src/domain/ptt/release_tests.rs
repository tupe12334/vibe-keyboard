use std::time::Instant;

use crate::domain::ptt::support::{ok, setup, start_hold, up_count, APP, SPACE};
use crate::domain::ptt::KeyEvent;

#[test]
fn shutdown_mid_hold_sends_exactly_one_key_up() {
    let (mut guard, log) = setup();
    start_hold(&mut guard, 1, Instant::now());
    guard.release_all();
    guard.release_all();
    assert_eq!(up_count(&log), 1);
    assert_eq!(
        log.borrow().last(),
        Some(&(
            APP,
            KeyEvent {
                code: SPACE,
                flags: 0,
                down: false,
                repeat: false
            }
        ))
    );
}

#[test]
fn disconnect_mid_hold_lifts_the_key() {
    let (mut guard, log) = setup();
    let t0 = Instant::now();
    let generation = start_hold(&mut guard, 1, t0);
    guard.release_all();
    assert!(!guard.pump(generation, ok(APP), t0));
    assert_eq!(up_count(&log), 1);
}

#[test]
fn drop_mid_hold_lifts_the_key() {
    let (mut guard, log) = setup();
    start_hold(&mut guard, 1, Instant::now());
    drop(guard);
    assert_eq!(up_count(&log), 1);
}

#[test]
fn releasing_another_key_keeps_the_hold() {
    let (mut guard, log) = setup();
    let t0 = Instant::now();
    let generation = start_hold(&mut guard, 1, t0);
    guard.end(2);
    assert_eq!(up_count(&log), 0);
    assert!(guard.pump(generation, ok(APP), t0));
    guard.end(1);
    assert_eq!(up_count(&log), 1);
}
