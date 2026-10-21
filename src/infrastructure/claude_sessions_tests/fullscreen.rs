use super::*;

/// The toggle script in the steps for `app`. Empty when there is no toggle.
fn toggle_script(app: TerminalApp) -> String {
    full_screen_steps(app)
        .into_iter()
        .find_map(|step| match step {
            FullScreenStep::Toggle(script) => Some(script),
            FullScreenStep::Focus => None,
        })
        .unwrap_or_default()
}

#[test]
fn full_screen_focuses_before_it_toggles() {
    for app in [TerminalApp::ITerm, TerminalApp::Terminal] {
        let steps = full_screen_steps(app);
        assert_eq!(steps.len(), 2, "{app:?}");
        assert_eq!(steps[0], FullScreenStep::Focus, "{app:?}");
        assert!(matches!(steps[1], FullScreenStep::Toggle(_)), "{app:?}");
    }
}

#[test]
fn full_screen_toggles_the_iterm_front_window() {
    let script = toggle_script(TerminalApp::ITerm);
    assert!(script.contains(r#"tell process "iTerm2""#));
    assert!(script.contains("set w to window 1"));
    assert!(script.contains(r#"set value of attribute "AXFullScreen" of w to (not (value of attribute "AXFullScreen" of w))"#));
    assert!(script.contains("window 1"));
}

#[test]
fn full_screen_toggles_the_terminal_front_window() {
    let script = toggle_script(TerminalApp::Terminal);
    assert!(script.contains(r#"tell process "Terminal""#));
    assert!(script.contains(r#"set w to value of attribute "AXFocusedWindow""#));
    assert!(script.contains(r#"set value of attribute "AXFullScreen" of w to (not (value of attribute "AXFullScreen" of w))"#));
    assert!(!script.contains("window 1"));
    assert!(!script.contains("iTerm2"));
}

#[test]
fn other_apps_get_focus_only() {
    assert_eq!(
        full_screen_steps(TerminalApp::Other),
        vec![FullScreenStep::Focus]
    );
    assert!(toggle_script(TerminalApp::Other).is_empty());
}
