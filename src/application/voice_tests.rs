use super::*;
use crate::domain::actions::{ClaudeSession, SessionStatus, TerminalApp};
use crate::domain::hold::{Gesture, Held, HoldTracker};
use crate::domain::navigation::{NavigationStack, Screen};

const HOLD: Duration = Duration::from_millis(400);

fn voice_press(holds: &mut HoldTracker, t0: Instant) {
    holds.press(VOICE_KEY, t0, Held::Voice);
}

#[test]
fn hold_on_voice_key_starts_then_release_ends() {
    let t0 = Instant::now();
    let mut holds = HoldTracker::default();
    voice_press(&mut holds, t0);
    let started = holds.poll(t0 + HOLD, HOLD);
    assert_eq!(
        started,
        [Gesture::HoldStart {
            key: VOICE_KEY,
            held: Held::Voice
        }]
    );
    let ended = holds.release(VOICE_KEY, t0 + HOLD + Duration::from_millis(100), HOLD);
    assert_eq!(
        ended,
        [Gesture::HoldEnd {
            key: VOICE_KEY,
            held: Held::Voice
        }]
    );
}

#[test]
fn tap_on_voice_key_is_a_tap_and_never_a_hold() {
    let t0 = Instant::now();
    let mut holds = HoldTracker::default();
    voice_press(&mut holds, t0);
    assert!(holds.poll(t0 + Duration::from_millis(100), HOLD).is_empty());
    let released = holds.release(VOICE_KEY, t0 + Duration::from_millis(100), HOLD);
    assert_eq!(
        released,
        [Gesture::Tap {
            key: VOICE_KEY,
            held: Held::Voice
        }]
    );
}

#[test]
fn flash_shows_hold_until_its_deadline_then_expires_once() {
    let t0 = Instant::now();
    let mut flash = None;
    arm(&mut flash, t0);
    assert!(flash.is_some());
    assert!(!take_expired(
        &mut flash,
        t0 + FLASH - Duration::from_millis(1)
    ));
    assert!(flash.is_some());
    assert!(take_expired(&mut flash, t0 + FLASH));
    assert!(flash.is_none());
    assert!(!take_expired(&mut flash, t0 + FLASH * 2));
}

#[test]
fn voice_hold_is_muted_for_apps_that_take_no_text() {
    let session = |app| ClaudeSession {
        pid: 1,
        app,
        tty: "ttys009".into(),
        label: "s".into(),
        status: SessionStatus::Active,
        age: None,
        focused: false,
    };
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::SessionActions {
        session: session(TerminalApp::ITerm),
        page: 0,
    });
    assert!(holds_voice(nav.current(), VOICE_KEY));
    assert!(!holds_voice(nav.current(), 14));
    nav.back();
    nav.push(Screen::SessionActions {
        session: session(TerminalApp::Other),
        page: 0,
    });
    assert!(!holds_voice(nav.current(), VOICE_KEY));
}
