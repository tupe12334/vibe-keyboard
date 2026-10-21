use super::{confirm_action, exit_action, open_confirm_exit, EXIT_TEXT};
use crate::application::key_action::Action;
use crate::application::prompt_return::{back_to_sessions, leave_after_send};
use crate::domain::actions::{ClaudeSession, SessionStatus, TerminalApp};
use crate::domain::navigation::{exit_on_page, NavigationStack, Screen};

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

/// Sessions list, then the session page, then the exit confirm on top.
fn confirm_stack(app: TerminalApp) -> NavigationStack {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::ClaudeSessions {
        sessions: vec![session(app)],
        page: 0,
    });
    nav.push(Screen::SessionActions {
        session: session(app),
        page: 0,
    });
    open_confirm_exit(&mut nav, session(app));
    nav
}

#[test]
fn exit_pushes_the_confirm_screen_above_the_session() {
    let nav = confirm_stack(TerminalApp::ITerm);
    assert!(matches!(nav.current(), Screen::ConfirmExit { .. }));
}

#[test]
fn cancel_on_key_11_returns_to_the_session_page() {
    let mut nav = confirm_stack(TerminalApp::ITerm);
    nav.back();
    assert!(matches!(nav.current(), Screen::SessionActions { .. }));
}

#[test]
fn cancel_on_key_12_returns_to_the_session_page() {
    let mut nav = confirm_stack(TerminalApp::ITerm);
    nav.out();
    assert!(matches!(nav.current(), Screen::SessionActions { .. }));
}

#[test]
fn confirm_sends_exactly_exit_to_the_session() {
    let s = session(TerminalApp::ITerm);
    let Action::SendPrompt { pid, tty, text } = confirm_action(13, &s) else {
        panic!("confirm must send a prompt");
    };
    assert_eq!((pid, tty.as_str(), text.as_str()), (7, "ttys009", "/exit"));
    assert_eq!(EXIT_TEXT, "/exit");
}

#[test]
fn cancel_keys_send_nothing() {
    let s = session(TerminalApp::ITerm);
    assert!(matches!(confirm_action(11, &s), Action::None));
    assert!(matches!(confirm_action(12, &s), Action::None));
}

#[test]
fn muted_apps_never_offer_or_send_exit() {
    let s = session(TerminalApp::Other);
    assert!(matches!(exit_action(&s), Action::None));
    assert!(matches!(confirm_action(13, &s), Action::None));
}

#[test]
fn exit_is_on_the_last_action_page_only() {
    assert!(exit_on_page(0, 0));
    assert!(exit_on_page(0, 8));
    assert!(!exit_on_page(0, 9));
    assert!(exit_on_page(1, 9));
}

#[test]
fn confirm_then_back_lands_on_the_sessions_list() {
    let mut nav = confirm_stack(TerminalApp::ITerm);
    assert!(leave_after_send(&nav, "ttys009", true));
    back_to_sessions(&mut nav);
    assert!(matches!(nav.current(), Screen::ClaudeSessions { .. }));
}

#[test]
fn a_failed_send_keeps_the_confirm_screen() {
    let nav = confirm_stack(TerminalApp::ITerm);
    assert!(!leave_after_send(&nav, "ttys009", false));
}
