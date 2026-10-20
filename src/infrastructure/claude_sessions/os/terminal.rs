//! osascript calls that focus a session tab, type a prompt into it, or open a new session window.

use super::front_tty::settle_on;
use super::{FOCUS_TIMEOUT, PS_TIMEOUT};
use crate::domain::actions::NewSessionTerminal;
use crate::infrastructure::claude_sessions::{
    focus_scripts, focused_bundle, is_frontmost, is_session_on_tty, is_vscode_session,
    new_session_script, send_text_scripts,
};
use crate::infrastructure::frontmost::frontmost_bundle_id;
use crate::infrastructure::process::run;
use std::time::{Duration, Instant};

/// How long to wait for a terminal to become frontmost after `open -b`.
const FRONT_WAIT: Duration = Duration::from_millis(500);
const FRONT_POLL: Duration = Duration::from_millis(50);

/// Select the terminal tab for `tty`. Tries iTerm2, then Terminal.app. Activates VS Code only if the
/// session runs under it. True only when the tab is found, its app is frontmost, and that tab is the
/// front session (two iTerm2 windows share one bundle id). Blocks; call it from a thread.
pub fn focus_session(tty: &str) -> bool {
    let Some(scripts) = focus_scripts(tty) else {
        return false;
    };
    let found = scripts.iter().enumerate().find_map(|(index, script)| {
        let out = run("osascript", &["-e", script], FOCUS_TIMEOUT)?;
        (out.trim() == "found").then_some(index)
    });
    let Some(index) = found else {
        // Open VS Code only for a session that runs under it. Others have no app to bring forward.
        if runs_in_vscode(tty) {
            run("open", &["-b", "com.microsoft.VSCode"], FOCUS_TIMEOUT);
        }
        return false;
    };
    focused_bundle(index).is_some_and(ensure_frontmost) && settle_on(index, tty)
}

/// True when the `claude` on `tty` runs under VS Code, per a fresh `ps`.
fn runs_in_vscode(tty: &str) -> bool {
    run("ps", &["-Ao", "pid=,ppid=,tty=,comm="], PS_TIMEOUT)
        .is_some_and(|ps| is_vscode_session(&ps, tty))
}

/// True when `bundle` is frontmost. If not, asks `LaunchServices` to open it (works from a
/// background app) and polls for up to `FRONT_WAIT`. Logs the frontmost app when it fails.
fn ensure_frontmost(bundle: &str) -> bool {
    let mut front = frontmost_bundle_id();
    if is_frontmost(front.as_deref(), bundle) {
        return true;
    }
    run("open", &["-b", bundle], FOCUS_TIMEOUT);
    let deadline = Instant::now() + FRONT_WAIT;
    loop {
        front = frontmost_bundle_id();
        if is_frontmost(front.as_deref(), bundle) {
            return true;
        }
        if Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(FRONT_POLL);
    }
    tracing::warn!(
        "claude sessions: {bundle} not frontmost after focus; frontmost is {}",
        front.as_deref().unwrap_or("unknown")
    );
    false
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
