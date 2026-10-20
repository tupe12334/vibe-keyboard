use super::*;
use std::cell::Cell;

#[test]
fn focus_failure_starts_no_push_to_talk() {
    let started = Cell::new(false);
    let ok = dictate_once_focused(|| false, || started.set(true));
    assert!(!ok);
    assert!(!started.get());
}

#[test]
fn focus_success_starts_push_to_talk_once() {
    let starts = Cell::new(0);
    let ok = dictate_once_focused(|| true, || starts.set(starts.get() + 1));
    assert!(ok);
    assert_eq!(starts.get(), 1);
}
