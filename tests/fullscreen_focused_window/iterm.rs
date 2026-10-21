//! Opens the two test windows in iTerm2, runs the app's own FULL SCREEN toggle, and closes them.
use std::process::Command;

use vibe_keyboard::domain::actions::TerminalApp;
use vibe_keyboard::infrastructure::claude_sessions::{full_screen_steps, FullScreenStep};

pub const FIRST: &str = "vk-ui-first";
pub const SECOND: &str = "vk-ui-second";

/// Runs `script` with osascript and returns its trimmed stdout. Panics with stderr on failure.
pub fn osa(script: &str) -> String {
    let out = Command::new("osascript").arg("-e").arg(script).output();
    let out = out.unwrap_or_else(|err| panic!("osascript did not start: {err}"));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "osascript failed: {}", err.trim());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// The toggle script the app runs for FULL SCREEN on iTerm2, read from the app's own steps.
pub fn app_toggle_script() -> String {
    let toggle = full_screen_steps(TerminalApp::ITerm)
        .into_iter()
        .find_map(|step| match step {
            FullScreenStep::Toggle(script) => Some(script),
            FullScreenStep::Focus => None,
        });
    match toggle {
        Some(script) => script,
        None => panic!("FULL SCREEN has no iTerm2 toggle step"),
    }
}

/// Opens two iTerm2 windows, then raises and focuses the second one.
pub fn open_windows() {
    osa(&format!(
        r#"tell application "iTerm2"
activate
set a to (create window with default profile)
tell current session of a to set name to "{FIRST}"
set b to (create window with default profile)
tell current session of b to set name to "{SECOND}"
select b
end tell"#
    ));
}

/// Closes the test windows when dropped. Never panics: a panic during unwinding aborts the test.
pub struct CloseWindows;

impl Drop for CloseWindows {
    fn drop(&mut self) {
        let script = r#"tell application "iTerm2"
repeat with w in (windows as list)
set n to name of current session of w
if n contains "vk-ui-" then close w
end repeat
end tell"#;
        let _ = Command::new("osascript").arg("-e").arg(script).output();
    }
}
