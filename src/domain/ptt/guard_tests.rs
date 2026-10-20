use std::time::Instant;

use crate::domain::ptt::support::{ok, setup, start_hold, APP, SPACE};
use crate::domain::ptt::Probe;

#[test]
fn second_hold_is_ignored_while_one_is_active() {
    let (mut guard, log) = setup();
    let t0 = Instant::now();
    start_hold(&mut guard, 1, t0);
    let second = guard.begin(2);
    assert!(!guard.start(second, 2, SPACE, 0, ok(APP), t0));
    let downs = log.borrow().iter().filter(|(_, e)| e.down).count();
    assert_eq!(downs, 1);
}

#[test]
fn start_is_refused_without_focus_or_trust() {
    let (mut guard, log) = setup();
    let now = Instant::now();
    let no_focus = guard.begin(1);
    let no_front = Probe {
        front: None,
        trusted: true,
    };
    assert!(!guard.start(no_focus, 1, SPACE, 0, no_front, now));
    let untrusted = guard.begin(1);
    let no_trust = Probe {
        front: Some(APP),
        trusted: false,
    };
    assert!(!guard.start(untrusted, 1, SPACE, 0, no_trust, now));
    assert!(log.borrow().is_empty());
}

#[test]
fn stale_begin_cannot_start_after_its_key_up() {
    let (mut guard, log) = setup();
    let old = guard.begin(1);
    guard.end(1);
    assert!(!guard.start(old, 1, SPACE, 0, ok(APP), Instant::now()));
    assert!(log.borrow().is_empty());
}

#[test]
fn old_generation_cannot_start_or_repeat() {
    let (mut guard, log) = setup();
    let t0 = Instant::now();
    let first = guard.begin(1);
    let second = guard.begin(1);
    assert!(!guard.start(first, 1, SPACE, 0, ok(APP), t0));
    assert!(guard.start(second, 1, SPACE, 0, ok(APP), t0));
    assert!(!guard.pump(first, t0));
    assert_eq!(log.borrow().len(), 1);
}
