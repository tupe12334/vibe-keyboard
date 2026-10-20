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
fn two_keys_track_independently() {
    let t0 = Instant::now();
    let mut holds = HoldTracker::default();
    holds.press(
        1,
        t0,
        Held::Session {
            tty: "ttys001".into(),
        },
    );
    holds.press(
        2,
        t0 + Duration::from_millis(300),
        Held::Session {
            tty: "ttys002".into(),
        },
    );
    // Key 1 reaches the threshold first. Key 2 is still short.
    let started = holds.poll(t0 + HOLD, HOLD);
    assert_eq!(
        started,
        vec![Gesture::HoldStart {
            key: 1,
            held: Held::Session {
                tty: "ttys001".into()
            }
        }]
    );
    // Key 2 comes up short, so it reports nothing.
    let gestures = holds.release(2, t0 + Duration::from_millis(500), HOLD);
    assert_eq!(
        gestures,
        vec![Gesture::Tap {
            key: 2,
            held: Held::Session {
                tty: "ttys002".into()
            }
        }]
    );
    // Key 1 ends its own hold.
    let ended = holds.release(1, t0 + Duration::from_millis(900), HOLD);
    assert!(matches!(
        ended.as_slice(),
        [Gesture::HoldEnd { key: 1, .. }]
    ));
}

#[test]
fn next_deadline_skips_keys_already_holding() {
    let t0 = Instant::now();
    let mut holds = HoldTracker::default();
    assert_eq!(holds.next_deadline(t0, HOLD), None);
    holds.press(1, t0, tap_held());
    holds.press(2, t0 + Duration::from_millis(100), tap_held());
    assert_eq!(
        holds.next_deadline(t0 + Duration::from_millis(50), HOLD),
        Some(Duration::from_millis(350))
    );
    holds.poll(t0 + HOLD, HOLD);
    assert_eq!(
        holds.next_deadline(t0 + HOLD, HOLD),
        Some(Duration::from_millis(100))
    );
    // Past the deadline the wait is zero, not negative.
    assert_eq!(
        holds.next_deadline(t0 + HOLD * 3, HOLD),
        Some(Duration::ZERO)
    );
}
