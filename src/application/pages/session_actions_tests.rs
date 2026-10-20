use super::keys::voice_key;
use super::open_session_actions;
use crate::application::voice::{tap_log, VOICE_LABEL};
use crate::domain::actions::{ClaudeSession, SessionStatus, TerminalApp};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::images::label_image;

#[test]
fn voice_key_never_names_the_voice_command() {
    let inset = crate::domain::profile::KeyInset::default();
    for typed in [true, false] {
        let (_, action) = voice_key(typed, &inset);
        assert!(!action.name.contains("/voice"));
        assert!(!action.description.contains("/voice"));
    }
}

#[test]
fn voice_key_is_muted_for_apps_that_take_no_text() {
    let inset = crate::domain::profile::KeyInset::default();
    let (_, typed) = voice_key(true, &inset);
    let (_, other) = voice_key(false, &inset);
    assert_ne!(typed.description, other.description);
    assert!(other.description.contains("no typed text"));
}

#[test]
fn selecting_a_session_focuses_its_tab_then_opens_actions() {
    let session = ClaudeSession {
        pid: 1,
        app: TerminalApp::ITerm,
        tty: "ttys009".into(),
        label: "s".into(),
        status: SessionStatus::Active,
        age: None,
        focused: false,
    };
    let mut nav = NavigationStack::new(0, 1);
    let mut focused = Vec::new();
    open_session_actions(&mut nav, session, |tty| focused.push(tty.to_string()));
    assert_eq!(focused, ["ttys009"]);
    assert!(matches!(
        nav.current(),
        Screen::SessionActions { page: 0, .. }
    ));
}

#[test]
fn voice_key_tile_draws_the_tap_label_not_bare_voice() {
    let inset = crate::domain::profile::KeyInset::default();
    let (tile, _) = voice_key(true, &inset);
    assert_eq!(VOICE_LABEL, "VOICE TAP=SEND");
    assert_ne!(tile, label_image("VOICE", &inset));
    assert_eq!(tile, label_image(VOICE_LABEL, &inset));
}

#[test]
fn tap_log_names_the_pid_that_got_enter() {
    assert_eq!(tap_log(4242), "push to talk: tap sent Enter to pid 4242");
}
