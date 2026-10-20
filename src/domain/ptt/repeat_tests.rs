use std::time::{Duration, Instant};

use crate::domain::ptt::support::{setup, start_hold, up_count, APP};
use crate::domain::ptt::WATCHDOG;

#[test]
fn one_target_pid_for_key_down_repeats_and_key_up() {
    let (mut guard, log) = setup();
    let t0 = Instant::now();
    let generation = start_hold(&mut guard, 1, t0);
    assert!(guard.pump(generation, t0));
    guard.end(1);
    let events = log.borrow();
    assert_eq!(events.len(), 3);
    assert!(events.iter().all(|(pid, _)| *pid == APP));
    assert!(events[0].1.down && !events[0].1.repeat);
    assert!(events[1].1.down && events[1].1.repeat);
    assert!(!events[2].1.down);
}

#[test]
fn beats_count_repeats_and_keep_the_widest_gap() {
    let (mut guard, _log) = setup();
    let t0 = Instant::now();
    let generation = start_hold(&mut guard, 1, t0);
    guard.touch(t0 + Duration::from_millis(400));
    assert!(guard.pump(generation, t0 + Duration::from_millis(30)));
    assert!(guard.pump(generation, t0 + Duration::from_millis(130)));
    assert!(guard.pump(generation, t0 + Duration::from_millis(160)));
    let beats = guard.hold.as_ref().map(|hold| hold.beats);
    assert_eq!(beats.map(|b| b.count), Some(3));
    assert_eq!(beats.map(|b| b.max_gap), Some(Duration::from_millis(100)));
}

#[test]
fn watchdog_stops_repeats_when_the_app_goes_quiet() {
    let (mut guard, log) = setup();
    let t0 = Instant::now();
    let generation = start_hold(&mut guard, 1, t0);
    guard.touch(t0 + Duration::from_millis(100));
    assert!(guard.pump(generation, t0 + Duration::from_millis(150)));
    let late = t0 + Duration::from_millis(100) + WATCHDOG * 2;
    assert!(!guard.pump(generation, late));
    assert_eq!(up_count(&log), 1);
}
