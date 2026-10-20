use super::{parse_selected_tty, selected_tty_script, set_focus, terminal_for_bundle};
use crate::domain::actions::{ClaudeSession, SessionStatus, TerminalApp};

fn session(app: TerminalApp, tty: &str) -> ClaudeSession {
    ClaudeSession {
        pid: 1,
        app,
        tty: tty.into(),
        label: "s".into(),
        status: SessionStatus::Active,
        age: None,
        focused: false,
    }
}

#[test]
fn maps_only_iterm_and_terminal_bundles() {
    assert_eq!(
        terminal_for_bundle("com.googlecode.iterm2"),
        Some(TerminalApp::ITerm)
    );
    assert_eq!(
        terminal_for_bundle("com.apple.Terminal"),
        Some(TerminalApp::Terminal)
    );
    assert_eq!(terminal_for_bundle("com.microsoft.VSCode"), None);
}

#[test]
fn other_apps_have_no_tab_script() {
    assert!(selected_tty_script(TerminalApp::Other).is_none());
    assert!(selected_tty_script(TerminalApp::ITerm).is_some_and(|s| s.contains("iTerm2")));
    assert!(selected_tty_script(TerminalApp::Terminal).is_some_and(|s| s.contains("Terminal")));
}

#[test]
fn parses_device_path_and_rejects_the_rest() {
    assert_eq!(parse_selected_tty("/dev/ttys003\n"), Some("ttys003".into()));
    assert_eq!(parse_selected_tty(""), None);
    assert_eq!(parse_selected_tty("ttys003"), None);
    assert_eq!(parse_selected_tty("/dev/null"), None);
}

#[test]
fn focuses_only_the_session_on_that_app_and_tty() {
    let mut sessions = vec![
        session(TerminalApp::ITerm, "ttys001"),
        session(TerminalApp::Terminal, "ttys001"),
        session(TerminalApp::Terminal, "ttys002"),
    ];
    set_focus(&mut sessions, Some((TerminalApp::Terminal, "ttys001")));
    let focused: Vec<bool> = sessions.iter().map(|s| s.focused).collect();
    assert_eq!(focused, [false, true, false]);
}

#[test]
fn no_focus_clears_every_session() {
    let mut sessions = vec![session(TerminalApp::ITerm, "ttys001")];
    set_focus(&mut sessions, Some((TerminalApp::ITerm, "ttys001")));
    set_focus(&mut sessions, None);
    assert!(sessions.iter().all(|s| !s.focused));
}
