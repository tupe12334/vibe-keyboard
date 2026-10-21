//! The `AppleScript` that opens a new Claude Code window. Fixed text only, no user input.

use crate::domain::actions::NewSessionTerminal;

/// `AppleScript` that opens a new window running `claude` in the home folder. iTerm2 types the
/// command into the new window's shell, so the user's own PATH and profile apply.
const ITERM_NEW_SESSION: &str = r#"tell application "iTerm2"
  activate
  create window with default profile
  tell current session of current window to write text "claude"
end tell"#;

/// `AppleScript` for Terminal.app. `do script` opens a new window and runs `claude` in it.
const TERMINAL_NEW_SESSION: &str = r#"tell application "Terminal"
  activate
  do script "claude"
end tell"#;

/// `AppleScript` that makes the new iTerm2 window full screen. The delay lets the window reach
/// System Events first.
const ITERM_FULL_SCREEN: &str = r#"delay 0.5
tell application "System Events"
  tell process "iTerm2"
    set value of attribute "AXFullScreen" of window 1 to true
  end tell
end tell"#;

/// `AppleScript` that makes the new Terminal.app window full screen. Same delay as iTerm2.
const TERMINAL_FULL_SCREEN: &str = r#"delay 0.5
tell application "System Events"
  tell process "Terminal"
    set value of attribute "AXFullScreen" of window 1 to true
  end tell
end tell"#;

/// The script that opens a new Claude Code window in `app`, made full screen when `fullscreen`.
/// Fixed text, so no user input is ever put into it.
pub fn new_session_script(app: NewSessionTerminal, fullscreen: bool) -> String {
    let (open, full) = match app {
        NewSessionTerminal::ITerm => (ITERM_NEW_SESSION, ITERM_FULL_SCREEN),
        NewSessionTerminal::Terminal => (TERMINAL_NEW_SESSION, TERMINAL_FULL_SCREEN),
    };
    if fullscreen {
        format!("{open}\n{full}")
    } else {
        open.into()
    }
}

#[cfg(test)]
mod tests {
    use super::new_session_script;
    use crate::domain::actions::NewSessionTerminal;

    #[test]
    fn new_session_script_runs_claude_in_the_chosen_app() {
        let iterm = new_session_script(NewSessionTerminal::ITerm, false);
        assert!(iterm.contains(r#"tell application "iTerm2""#));
        assert!(iterm.contains(r#"write text "claude""#));
        assert!(!iterm.contains("Terminal"));
        let terminal = new_session_script(NewSessionTerminal::Terminal, false);
        assert!(terminal.contains(r#"tell application "Terminal""#));
        assert!(terminal.contains(r#"do script "claude""#));
        assert!(!terminal.contains("iTerm2"));
    }

    #[test]
    fn fullscreen_runs_after_the_window_opens() {
        let iterm = new_session_script(NewSessionTerminal::ITerm, true);
        let created = iterm.find("create window");
        let full = iterm.find("AXFullScreen");
        assert!(
            created.is_some() && created < full,
            "window before full screen"
        );
        assert!(iterm.contains(r#"tell process "iTerm2""#));
        let terminal = new_session_script(NewSessionTerminal::Terminal, true);
        assert!(terminal.contains(r#"tell process "Terminal""#));
        assert!(!terminal.contains("iTerm2"));
    }

    #[test]
    fn default_session_stays_out_of_full_screen() {
        let iterm = new_session_script(NewSessionTerminal::ITerm, false);
        assert!(!iterm.contains("AXFullScreen"));
    }
}
