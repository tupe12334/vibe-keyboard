//! The steps of the FULL SCREEN key, in order. Pure: the osascript runs live in `claude_sessions/os/fullscreen.rs`.

use crate::domain::actions::TerminalApp;

/// One step of the FULL SCREEN key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FullScreenStep {
    /// Focus the session's tab first, so the toggle lands on its window.
    Focus,
    /// Run this `AppleScript` to toggle full screen on the front window.
    Toggle(String),
}

/// Toggles full screen on iTerm2's front window. Fixed text, no user input.
const ITERM_TOGGLE: &str = r#"tell application "System Events" to tell process "iTerm2" to set value of attribute "AXFullScreen" of window 1 to (not (value of attribute "AXFullScreen" of window 1))"#;

/// Toggles full screen on Terminal.app's front window. Fixed text, no user input.
const TERMINAL_TOGGLE: &str = r#"tell application "System Events" to tell process "Terminal" to set value of attribute "AXFullScreen" of window 1 to (not (value of attribute "AXFullScreen" of window 1))"#;

/// The steps for `app`. Focus always comes first. Other apps get focus only, since no toggle is known.
pub fn full_screen_steps(app: TerminalApp) -> Vec<FullScreenStep> {
    let toggle = match app {
        TerminalApp::ITerm => ITERM_TOGGLE,
        TerminalApp::Terminal => TERMINAL_TOGGLE,
        TerminalApp::Other => return vec![FullScreenStep::Focus],
    };
    vec![FullScreenStep::Focus, FullScreenStep::Toggle(toggle.into())]
}
