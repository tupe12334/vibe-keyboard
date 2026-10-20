use std::time::{Duration, Instant};

use crate::domain::ptt::release::key_up_summary;
use crate::domain::ptt::repeat::Beats;
use crate::domain::ptt::support::{setup, start_hold, up_count, APP, SPACE};
use crate::domain::ptt::{KeyEvent, Release};

#[test]
fn shutdown_mid_hold_sends_exactly_one_key_up() {
    let (mut guard, log) = setup();
    start_hold(&mut guard, 1, Instant::now());
    guard.release_all(Release::Shutdown);
    guard.release_all(Release::Shutdown);
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
    guard.release_all(Release::Shutdown);
    assert!(!guard.pump(generation, t0));
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
    assert!(guard.pump(generation, t0));
    guard.end(1);
    assert_eq!(up_count(&log), 1);
}

#[test]
fn key_up_after_a_screen_change_release_sends_no_second_key_up() {
    let (mut guard, log) = setup();
    start_hold(&mut guard, 1, Instant::now());
    guard.release_all(Release::Shutdown);
    guard.end(1);
    assert_eq!(up_count(&log), 1);
}

#[test]
fn every_reason_has_its_log_label() {
    assert_eq!(Release::Key.label(), "released");
    assert_eq!(Release::Moved { to: Some(7) }.label(), "moved to pid 7");
    assert_eq!(
        Release::Moved { to: None }.label(),
        "moved, no frontmost app"
    );
    assert_eq!(Release::Untrusted.label(), "untrusted");
    assert_eq!(Release::Watchdog.label(), "watchdog");
    assert_eq!(Release::Screen.label(), "screen change");
    assert_eq!(Release::Disconnected.label(), "device lost");
    assert_eq!(Release::Shutdown.label(), "shutdown");
}

#[test]
fn key_up_summary_has_count_reason_and_widest_gap() {
    let t0 = Instant::now();
    let mut beats = Beats::new(t0);
    beats.beat(t0 + Duration::from_millis(30));
    beats.beat(t0 + Duration::from_millis(130));
    assert_eq!(
        key_up_summary(beats, Release::Moved { to: Some(7) }),
        "key up after 2 repeats: moved to pid 7, max repeat gap 100 ms"
    );
}
