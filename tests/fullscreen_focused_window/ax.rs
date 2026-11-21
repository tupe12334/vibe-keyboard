//! Reads the iTerm2 windows' Accessibility state (title, full-screen) through System Events.
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::iterm::osa;

const WAIT: Duration = Duration::from_secs(15);

/// One line per iTerm2 AX window: `title=true|false` (full-screen state), in AX order.
pub fn ax_windows() -> String {
    osa(r#"tell application "System Events"
tell process "iTerm2"
set out to ""
repeat with w in windows
set out to out & (value of attribute "AXTitle" of w as text) & "=" & (value of attribute "AXFullScreen" of w as text) & linefeed
end repeat
return out
end tell
end tell"#)
}

/// The AX title of the focused iTerm2 window.
pub fn focused_title() -> String {
    osa(r#"tell application "System Events"
tell process "iTerm2"
return value of attribute "AXTitle" of (value of attribute "AXFocusedWindow")
end tell
end tell"#)
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
