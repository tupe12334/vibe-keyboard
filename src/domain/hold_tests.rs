use super::*;
use crate::domain::profile::Action;

const HOLD: Duration = Duration::from_millis(400);

fn tap_held() -> Held {
    Held::Action {
        short: Action::GoToPage { page: 1 },
        hold: Some(Action::PushToTalk { key: None }),
    }
}

#[test]
fn short_press_is_a_tap_on_release() {
    let t0 = Instant::now();
    let mut holds = HoldTracker::default();
    holds.press(3, t0, tap_held());
    assert!(holds.poll(t0 + Duration::from_millis(100), HOLD).is_empty());
    let gestures = holds.release(3, t0 + Duration::from_millis(100), HOLD);
    assert_eq!(
        gestures,
        vec![Gesture::Tap {
            key: 3,
            held: tap_held()
        }]
    );
}

#[test]
fn long_press_starts_hold_on_poll_then_ends_on_release() {
    let t0 = Instant::now();
    let mut holds = HoldTracker::default();
    holds.press(3, t0, tap_held());
    let started = holds.poll(t0 + HOLD, HOLD);
    assert_eq!(
        started,
        vec![Gesture::HoldStart {
            key: 3,
            held: tap_held()
        }]
    );
    // A second poll does not start the hold again.
    assert!(holds.poll(t0 + HOLD * 2, HOLD).is_empty());
    let ended = holds.release(3, t0 + HOLD * 2, HOLD);
    assert_eq!(
        ended,
        vec![Gesture::HoldEnd {
            key: 3,
            held: tap_held()
        }]
    );
}

#[test]
fn release_past_threshold_without_poll_reports_start_then_end() {
    let t0 = Instant::now();
    let mut holds = HoldTracker::default();
    holds.press(7, t0, tap_held());
    let gestures = holds.release(7, t0 + HOLD, HOLD);
    assert_eq!(
        gestures,
        vec![
            Gesture::HoldStart {
                key: 7,
                held: tap_held()
            },
            Gesture::HoldEnd {
                key: 7,
                held: tap_held()
            },
        ]
    );
}

#[test]
fn repeat_press_keeps_the_first_time() {
    let t0 = Instant::now();
    let mut holds = HoldTracker::default();
    holds.press(4, t0, tap_held());
    holds.press(4, t0 + HOLD, tap_held());
    assert_eq!(holds.poll(t0 + HOLD, HOLD).len(), 1);
}

#[test]
fn release_of_unknown_key_does_nothing() {
    let mut holds = HoldTracker::default();
    assert!(holds.release(9, Instant::now(), HOLD).is_empty());
}
