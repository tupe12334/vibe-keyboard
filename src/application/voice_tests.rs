use super::*;
use crate::domain::actions::{ClaudeSession, SessionStatus, TerminalApp};
use crate::domain::hold::{Gesture, Held, HoldTracker};
use crate::domain::navigation::{NavigationStack, Screen};
use std::time::{Duration, Instant};

const HOLD: Duration = Duration::from_millis(400);

fn voice_press(holds: &mut HoldTracker, t0: Instant) {
    holds.press(VOICE_KEY, t0, Held::Voice);
}

fn session(app: TerminalApp) -> ClaudeSession {
    ClaudeSession {
        pid: 7,
        app,
        tty: "ttys009".into(),
        label: "s".into(),
        status: SessionStatus::Active,
        age: None,
        focused: false,
        remote_control: false,
    }
}

fn session_page(app: TerminalApp) -> NavigationStack {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::SessionActions {
        session: session(app),
        page: 0,
    });
    nav
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
fn tap_on_voice_key_sends_enter_to_the_session() {
    let nav = session_page(TerminalApp::ITerm);
    let Some(Action::SendPrompt { pid, tty, text }) = enter_action(nav.current()) else {
        panic!("tap on voice must send");
    };
    assert_eq!((pid, tty.as_str(), text.as_str()), (7, "ttys009", ""));
}

#[test]
fn voice_key_sends_nothing_for_muted_apps_or_other_screens() {
    assert!(enter_action(session_page(TerminalApp::Other).current()).is_none());
    assert!(enter_action(NavigationStack::new(0, 1).current()).is_none());
}

#[test]
fn voice_hold_is_muted_for_apps_that_take_no_text() {
    let nav = session_page(TerminalApp::ITerm);
    assert!(holds_voice(nav.current(), VOICE_KEY));
    assert!(!holds_voice(nav.current(), 14));
    assert!(!holds_voice(
        session_page(TerminalApp::Other).current(),
        VOICE_KEY
    ));
}
