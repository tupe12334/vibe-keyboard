//! osascript calls that focus a session tab or type a prompt into it.

use super::front_tty::{find_tab, settle_on};
use super::{FOCUS_TIMEOUT, PS_TIMEOUT};
use crate::infrastructure::claude_sessions::{
    focus_scripts, focused_bundle, is_frontmost, is_vscode_session, resolve_session_pid,
    send_text_scripts,
};
use crate::infrastructure::frontmost::frontmost_bundle_id;
use crate::infrastructure::process::Logging::WarnCodeOnly;
use crate::infrastructure::process::{run, run_with};
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
    let Some(index) = find_tab(&scripts) else {
        // Open VS Code only for a session that runs under it. Others have no app to bring forward.
        if runs_in_vscode(tty) {
            run("open", &["-b", "com.microsoft.VSCode"], FOCUS_TIMEOUT);
        }
        return false;
    };
    // One retry: the first select can lose the race with the app coming forward.
    let landed = || focused_bundle(index).is_some_and(ensure_frontmost) && settle_on(index, tty);
    landed() || (run("osascript", &["-e", &scripts[index]], FOCUS_TIMEOUT).is_some() && landed())
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

/// Type `text` and Enter into the session's tab. Tries iTerm2, then Terminal.app. When `pid` was
/// restarted, the `claude` now on `tty` gets the prompt. Skips when no `claude` runs on `tty`. True
/// only when a tab confirmed the send. Blocks until done, so call it from a thread.
pub fn send_prompt(pid: u32, tty: &str, text: &str) -> bool {
    let Some(scripts) = send_text_scripts(tty, text) else {
        return false;
    };
    let Some(live) = run("ps", &["-Ao", "pid=,tty=,comm="], PS_TIMEOUT)
        .and_then(|ps| resolve_session_pid(&ps, pid, tty))
    else {
        tracing::warn!("claude sessions: no claude session on {tty}; prompt skipped");
        return false;
    };
    if live != pid {
        tracing::info!("claude sessions: pid {pid} restarted as {live} on {tty}");
    }
    let sent = scripts.iter().any(|script| {
        let out = run_with("osascript", &["-e", script], FOCUS_TIMEOUT, WarnCodeOnly);
        out.is_some_and(|out| out.trim() == "found")
    });
    if !sent {
        tracing::warn!(
            "claude sessions: no iTerm2 or Terminal tab confirmed on {tty}; send may not have completed"
        );
    }
    sent
}

/// Run `script` from `new_session_script` to open a new session window. Blocks; call from a thread.
pub fn open_new_session(script: &str) {
    run("osascript", &["-e", script], FOCUS_TIMEOUT);
}
