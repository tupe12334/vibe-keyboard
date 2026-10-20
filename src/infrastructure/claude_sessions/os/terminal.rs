//! osascript calls that focus a session tab, type a prompt into it, or open a new session window.

use super::{run, FOCUS_TIMEOUT, PS_TIMEOUT};
use crate::domain::actions::NewSessionTerminal;
use crate::infrastructure::claude_sessions::{
    focus_scripts, is_session_on_tty, new_session_script, send_text_scripts,
};

/// Select the terminal tab for `tty`. Tries iTerm2, then Terminal.app. When neither has the tab
/// (VS Code and other terminals), it activates VS Code. Blocks until done, so call it from a thread.
pub fn focus_session(tty: &str) {
    let Some(scripts) = focus_scripts(tty) else {
        return;
    };
    let found = scripts.iter().any(|script| {
        run("osascript", &["-e", script], FOCUS_TIMEOUT).is_some_and(|out| out.trim() == "found")
    });
    if !found {
        run("open", &["-b", "com.microsoft.VSCode"], FOCUS_TIMEOUT);
    }
}

/// Type `text` and Enter into the session's tab. Tries iTerm2, then Terminal.app. Skips when `pid`
/// no longer runs `claude` on `tty`, because the tty may have been reused. Logs when no app
/// confirms the send. Returns true only when a tab confirmed it. Blocks until done, so call it from
/// a thread.
pub fn send_prompt(pid: u32, tty: &str, text: &str) -> bool {
    let Some(scripts) = send_text_scripts(tty, text) else {
        return false;
    };
    let still_there = run("ps", &["-Ao", "pid=,tty=,comm="], PS_TIMEOUT)
        .is_some_and(|ps| is_session_on_tty(&ps, pid, tty));
    if !still_there {
        tracing::warn!("claude sessions: pid {pid} no longer on {tty}; prompt skipped");
        return false;
    }
    let sent = scripts.iter().any(|script| {
        run("osascript", &["-e", script], FOCUS_TIMEOUT).is_some_and(|out| out.trim() == "found")
    });
    if !sent {
        tracing::warn!(
            "claude sessions: no iTerm2 or Terminal tab confirmed on {tty}; send may not have completed"
        );
    }
    sent
}

/// Open a new window in `app` that runs `claude`. Blocks until done, so call it from a thread.
pub fn open_new_session(app: NewSessionTerminal) {
    run("osascript", &["-e", new_session_script(app)], FOCUS_TIMEOUT);
}
