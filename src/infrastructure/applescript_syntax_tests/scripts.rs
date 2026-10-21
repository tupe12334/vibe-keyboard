//! The `AppleScript` text the app builds from fixed or pure input, collected for the syntax test.

use crate::application::actions::claude::OPEN_CLAUDE_SCRIPT;
use crate::application::actions::terminal::{terminal_in_path_script, OPEN_TERMINAL_SCRIPT};
use crate::domain::actions::{NewSessionTerminal, TerminalApp};
use crate::domain::hotkey::parse_hotkey;
use crate::infrastructure::claude_sessions::{
    focus_scripts, full_screen_steps, new_session_script, send_text_scripts, FullScreenStep,
    ITERM_FRONT_TTY, ITERM_TITLES, TERMINAL_FRONT_TTY, TERMINAL_TITLES,
};
use crate::infrastructure::focused_session::selected_tty_script;

const TTY: &str = "ttys003";

/// Labels the two scripts of a builder `<what> iTerm2` and `<what> Terminal`. A None builder gives
/// no entries, and the count check in the test then fails.
fn labelled(what: &str, built: Option<[String; 2]>) -> Vec<(String, String)> {
    let apps = ["iTerm2", "Terminal"];
    built
        .into_iter()
        .flatten()
        .zip(apps)
        .map(|(text, app)| (format!("{what} {app}"), text))
        .collect()
}

/// Every script the app builds from pure input, each with a label for the failure report.
pub(super) fn scripts() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (label, text) in [
        ("ITERM_TITLES", ITERM_TITLES),
        ("TERMINAL_TITLES", TERMINAL_TITLES),
        ("ITERM_FRONT_TTY", ITERM_FRONT_TTY),
        ("TERMINAL_FRONT_TTY", TERMINAL_FRONT_TTY),
        ("OPEN_CLAUDE_SCRIPT", OPEN_CLAUDE_SCRIPT),
        ("OPEN_TERMINAL_SCRIPT", OPEN_TERMINAL_SCRIPT),
    ] {
        out.push((label.to_string(), text.to_string()));
    }
    for app in [TerminalApp::ITerm, TerminalApp::Terminal] {
        for step in full_screen_steps(app) {
            if let FullScreenStep::Toggle(text) = step {
                out.push((format!("full screen toggle {app:?}"), text));
            }
        }
        out.extend(
            selected_tty_script(app)
                .map(|text| (format!("selected tty {app:?}"), text.to_string())),
        );
    }
    for app in [NewSessionTerminal::ITerm, NewSessionTerminal::Terminal] {
        for fullscreen in [false, true] {
            let text = new_session_script(app, fullscreen);
            out.push((format!("new session {app:?} fullscreen={fullscreen}"), text));
        }
    }
    out.extend(labelled("focus", focus_scripts(TTY)));
    for text in ["claude", r#"echo "hi" \ there"#] {
        out.extend(labelled(
            &format!("send {text}"),
            send_text_scripts(TTY, text),
        ));
    }
    for keys in ["cmd+k", "cmd+shift+return", "opt+left", "escape"] {
        // A parse error becomes text that osacompile rejects, so the test fails.
        let text = parse_hotkey(keys).map_or_else(|err| err, |hotkey| hotkey.applescript());
        out.push((format!("hotkey {keys}"), text));
    }
    out.push((
        "terminal in path (none)".to_string(),
        terminal_in_path_script(None),
    ));
    out.push((
        "terminal in path".to_string(),
        terminal_in_path_script(Some("/Users/me/dev")),
    ));
    out
}

/// Number of scripts `scripts` returns. Update it when a builder gains or loses one.
pub(super) const SCRIPT_COUNT: usize = 26;
