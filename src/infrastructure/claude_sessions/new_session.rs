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

/// The script that opens a new Claude Code window in `app`. Fixed text, so no user input is
/// ever put into it.
pub const fn new_session_script(app: NewSessionTerminal) -> &'static str {
    match app {
        NewSessionTerminal::ITerm => ITERM_NEW_SESSION,
        NewSessionTerminal::Terminal => TERMINAL_NEW_SESSION,
    }
}

#[cfg(test)]
mod tests {
    use super::new_session_script;
    use crate::domain::actions::NewSessionTerminal;

    #[test]
    fn new_session_script_runs_claude_in_the_chosen_app() {
        let iterm = new_session_script(NewSessionTerminal::ITerm);
        assert!(iterm.contains(r#"tell application "iTerm2""#));
        assert!(iterm.contains(r#"write text "claude""#));
        assert!(!iterm.contains("Terminal"));
        let terminal = new_session_script(NewSessionTerminal::Terminal);
        assert!(terminal.contains(r#"tell application "Terminal""#));
        assert!(terminal.contains(r#"do script "claude""#));
        assert!(!terminal.contains("iTerm2"));
    }
}
