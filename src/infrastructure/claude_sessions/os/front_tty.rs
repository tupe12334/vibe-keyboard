//! osascript reads of the tty shown in front, the wait for focus to land on it, and the BLINK
//! highlight. Excluded from the coverage gate; the rules live in `claude_sessions/focus_check.rs`
//! and `claude_sessions/highlight.rs`.

use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use tracing::warn;

use super::terminal::focus_session;
use crate::domain::actions::TerminalApp;
use crate::infrastructure::claude_sessions::{highlight_target, is_selected, session_moved};
use crate::infrastructure::process::run;

/// Hard limit for one read. Each read is a separate osascript call.
const READ_TIMEOUT: Duration = Duration::from_secs(2);
/// How long focus may take to land on the selected tty before dictation is refused.
const SETTLE: Duration = Duration::from_secs(1);
const POLL: Duration = Duration::from_millis(50);

/// `AppleScript` that prints the tty of iTerm2's current session. Empty when iTerm2 is not running.
pub const ITERM_FRONT_TTY: &str = r#"if application "iTerm2" is running then
  tell application "iTerm2"
    if (count of windows) is 0 then return ""
    return tty of current session of current tab of current window
  end tell
end if
return """#;

/// `AppleScript` that prints the tty of the selected tab in Terminal.app's front window.
pub const TERMINAL_FRONT_TTY: &str = r#"if application "Terminal" is running then
  tell application "Terminal"
    if (count of windows) is 0 then return ""
    return tty of selected tab of front window
  end tell
end if
return """#;

/// The tty shown in front by the terminal at `index` (0 iTerm2, 1 Terminal.app). None when unknown.
fn front_tty(index: usize) -> Option<String> {
    let script = [ITERM_FRONT_TTY, TERMINAL_FRONT_TTY].get(index)?;
    let out = run("osascript", &["-e", script], READ_TIMEOUT)?;
    Some(out.trim().to_string()).filter(|tty| !tty.is_empty())
}

/// Poll for up to `SETTLE` until the terminal at `index` shows `tty` in front. Logs a warning and
/// returns false when it never does, so the caller posts no key.
pub fn settle_on(index: usize, tty: &str) -> bool {
    let deadline = Instant::now() + SETTLE;
    loop {
        let got = front_tty(index);
        if is_selected(got.as_deref(), tty) {
            return true;
        }
        if Instant::now() >= deadline {
            warn!(
                "push to talk: selected session tty {tty} not frontmost (got {})",
                got.as_deref().unwrap_or("unknown")
            );
            return false;
        }
        thread::sleep(POLL);
    }
}

/// True when iTerm2's current session is no longer `tty`. Read once, for the push-to-talk probe.
pub fn iterm_moved_from(tty: &str) -> bool {
    session_moved(front_tty(0).as_deref(), tty)
}

/// Focus the session on `tty`, then ask iTerm2 for a fireworks highlight on that pane. Only iTerm2
/// reads the escape, so other apps get focus only. Blocks; call it from a thread.
pub fn blink_session(tty: &str, app: TerminalApp) {
    focus_session(tty);
    if app != TerminalApp::ITerm {
        return;
    }
    let Some((path, bytes)) = highlight_target(tty) else {
        return;
    };
    let written = OpenOptions::new()
        .write(true)
        .open(&path)
        .and_then(|mut device| device.write_all(bytes));
    if let Err(err) = written {
        warn!("claude sessions: highlight on {path} failed: {err}");
    }
}
