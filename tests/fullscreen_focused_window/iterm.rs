//! Drives real iTerm2 windows and reads their Accessibility state through osascript.
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use vibe_keyboard::domain::actions::TerminalApp;
use vibe_keyboard::infrastructure::claude_sessions::{full_screen_steps, FullScreenStep};

pub const FIRST: &str = "vk-ui-first";
pub const SECOND: &str = "vk-ui-second";
const WAIT: Duration = Duration::from_secs(15);

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
        "tell application \"iTerm2\"\n\
         activate\n\
         set a to (create window with default profile)\n\
         tell current session of a to set name to \"{FIRST}\"\n\
         set b to (create window with default profile)\n\
         tell current session of b to set name to \"{SECOND}\"\n\
         select b\n\
         end tell"
    ));
}

/// One line per iTerm2 AX window: `title=true|false` (full-screen state), in AX order.
pub fn ax_windows() -> String {
    osa("tell application \"System Events\" to tell process \"iTerm2\"\n\
         set out to \"\"\n\
         repeat with w in windows\n\
         set out to out & (value of attribute \"AXTitle\" of w as text) & \"=\" & (value of attribute \"AXFullScreen\" of w as text) & linefeed\n\
         end repeat\n\
         return out\n\
         end tell")
}

/// The AX title of the focused iTerm2 window.
pub fn focused_title() -> String {
    osa(
        "tell application \"System Events\" to tell process \"iTerm2\" to \
         return value of attribute \"AXTitle\" of (value of attribute \"AXFocusedWindow\")",
    )
}

/// Full-screen state of the window whose title contains `title`. None when no window matches.
pub fn fullscreen_of(lines: &str, title: &str) -> Option<bool> {
    lines
        .lines()
        .find(|line| line.contains(title))
        .map(|line| line.ends_with("=true"))
}

/// Polls the AX windows until `title` reaches `want`, or gives up after `WAIT`. Returns the last read.
pub fn wait_fullscreen(title: &str, want: bool) -> String {
    let start = Instant::now();
    loop {
        let lines = ax_windows();
        if fullscreen_of(&lines, title) == Some(want) || start.elapsed() > WAIT {
            return lines;
        }
        sleep(Duration::from_millis(500));
    }
}

/// Closes the test windows when dropped, pass or fail.
pub struct CloseWindows;

impl Drop for CloseWindows {
    fn drop(&mut self) {
        let _ = osa("tell application \"iTerm2\"\n\
             repeat with w in (windows as list)\n\
             if (name of current session of w) contains \"vk-ui-\" then close w\n\
             end repeat\n\
             end tell");
    }
}
