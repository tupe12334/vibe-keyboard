use std::time::{Duration, Instant};

use crate::domain::ptt::support::{ok, setup, start_hold, up_count, APP, OTHER};
use crate::domain::ptt::{Probe, WATCHDOG};

#[test]
fn one_target_pid_for_key_down_repeats_and_key_up() {
    let (mut guard, log) = setup();
    let t0 = Instant::now();
    let generation = start_hold(&mut guard, 1, t0);
    assert!(guard.pump(generation, ok(APP), t0));
    guard.end(1);
    let events = log.borrow();
    assert_eq!(events.len(), 3);
    assert!(events.iter().all(|(pid, _)| *pid == APP));
    assert!(events[0].1.down && !events[0].1.repeat);
    assert!(events[1].1.down && events[1].1.repeat);
    assert!(!events[2].1.down);
}

#[test]
fn focus_change_mid_hold_never_sends_space_to_the_new_app() {
    let (mut guard, log) = setup();
    let t0 = Instant::now();
    let generation = start_hold(&mut guard, 1, t0);
    assert!(!guard.pump(generation, ok(OTHER), t0));
    let events = log.borrow();
    assert!(events.iter().all(|(pid, _)| *pid != OTHER));
    assert_eq!(events.last().map(|(pid, _)| *pid), Some(APP));
    assert!(events.last().is_some_and(|(_, e)| !e.down));
}

#[test]
fn lost_accessibility_lifts_the_key_on_the_next_repeat() {
    let (mut guard, log) = setup();
    let t0 = Instant::now();
    let generation = start_hold(&mut guard, 1, t0);
    let untrusted = Probe {
        front: Some(APP),
        trusted: false,
    };
    assert!(!guard.pump(generation, untrusted, t0));
    assert_eq!(up_count(&log), 1);
}

#[test]
fn watchdog_stops_repeats_when_the_app_goes_quiet() {
    let (mut guard, log) = setup();
    let t0 = Instant::now();
    let generation = start_hold(&mut guard, 1, t0);
    guard.touch(t0 + Duration::from_millis(100));
    assert!(guard.pump(generation, ok(APP), t0 + Duration::from_millis(150)));
    let late = t0 + Duration::from_millis(100) + WATCHDOG * 2;
    assert!(!guard.pump(generation, ok(APP), late));
    assert_eq!(up_count(&log), 1);
}
