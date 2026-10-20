use std::time::{Duration, Instant};

use crate::domain::ptt::repeat::Beats;
use crate::domain::ptt::support::{setup, start_hold, up_count, APP};
use crate::domain::ptt::{deadline, next_index, WATCHDOG};

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

#[test]
fn deadlines_sit_on_the_grid_from_start() {
    let t0 = Instant::now();
    assert_eq!(deadline(t0, 1), t0 + Duration::from_millis(30));
    assert_eq!(deadline(t0, 4), t0 + Duration::from_millis(120));
}

#[test]
fn an_on_time_beat_waits_for_the_next_deadline() {
    let t0 = Instant::now();
    assert_eq!(next_index(t0, 1, t0 + Duration::from_millis(30)), 2);
    assert_eq!(next_index(t0, 2, t0 + Duration::from_millis(61)), 3);
}

#[test]
fn a_late_beat_posts_once_and_skips_the_missed_grid_points() {
    let t0 = Instant::now();
    // Woke at 130 ms for deadline 30: the 60, 90 and 120 ms points are skipped, not burst.
    let next = next_index(t0, 1, t0 + Duration::from_millis(130));
    assert_eq!(next, 5);
    assert_eq!(deadline(t0, next), t0 + Duration::from_millis(150));
}

#[test]
fn the_next_deadline_is_never_already_past() {
    let t0 = Instant::now();
    for ms in (0..1000).step_by(7) {
        let now = t0 + Duration::from_millis(ms);
        let next = next_index(t0, 1, now);
        assert!(deadline(t0, next) > now, "stale deadline at {ms} ms");
    }
}

#[test]
fn a_gap_over_fifty_ms_counts_as_late() {
    let t0 = Instant::now();
    let mut beats = Beats::new(t0);
    beats.beat(t0 + Duration::from_millis(30));
    beats.beat(t0 + Duration::from_millis(80));
    beats.beat(t0 + Duration::from_millis(160));
    assert_eq!(beats.late, 1);
    assert_eq!(beats.count, 3);
}
