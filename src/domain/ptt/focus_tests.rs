use std::time::Instant;

use crate::domain::ptt::focus::MoveCheck;
use crate::domain::ptt::support::{ok, setup, start_hold, up_count, APP, OTHER};
use crate::domain::ptt::Probe;

#[test]
fn one_odd_probe_does_not_end_the_hold() {
    let (mut guard, log) = setup();
    let generation = start_hold(&mut guard, 1, Instant::now());
    assert!(guard.observe(generation, ok(OTHER)));
    assert!(guard.observe(generation, ok(APP)));
    assert!(guard.pump(generation, Instant::now()));
    assert_eq!(up_count(&log), 0);
}

#[test]
fn two_probes_in_a_row_move_the_hold_and_lift_the_key_on_its_own_app() {
    let (mut guard, log) = setup();
    let generation = start_hold(&mut guard, 1, Instant::now());
    assert!(guard.observe(generation, ok(OTHER)));
    assert!(!guard.observe(generation, ok(OTHER)));
    assert_eq!(up_count(&log), 1);
    assert!(log.borrow().iter().all(|(pid, _)| *pid != OTHER));
    assert_eq!(log.borrow().last().map(|(pid, _)| *pid), Some(APP));
}

#[test]
fn a_probe_with_no_frontmost_app_counts_as_a_miss() {
    let (mut guard, log) = setup();
    let generation = start_hold(&mut guard, 1, Instant::now());
    let none = Probe {
        front: None,
        trusted: true,
    };
    assert!(guard.observe(generation, none));
    assert!(!guard.observe(generation, none));
    assert_eq!(up_count(&log), 1);
}

#[test]
fn lost_accessibility_lifts_the_key_at_once() {
    let (mut guard, log) = setup();
    let generation = start_hold(&mut guard, 1, Instant::now());
    let untrusted = Probe {
        front: Some(APP),
        trusted: false,
    };
    assert!(!guard.observe(generation, untrusted));
    assert_eq!(up_count(&log), 1);
}

#[test]
fn a_stale_or_ended_hold_ignores_probes() {
    let (mut guard, log) = setup();
    let generation = start_hold(&mut guard, 1, Instant::now());
    assert!(!guard.observe(generation + 1, ok(OTHER)));
    guard.end(1);
    assert!(!guard.observe(generation, ok(OTHER)));
    assert_eq!(up_count(&log), 1);
}

#[test]
fn move_check_resets_after_a_hit() {
    let mut check = MoveCheck::default();
    assert!(!check.observe(Some(OTHER), APP));
    assert!(!check.observe(Some(APP), APP));
    assert!(!check.observe(Some(OTHER), APP));
    assert!(check.observe(Some(OTHER), APP));
}
