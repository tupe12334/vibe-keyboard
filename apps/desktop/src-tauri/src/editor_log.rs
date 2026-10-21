//! Editor events for app.log. The editor console is hidden, so its errors and saves are logged here.

use vibe_keyboard::domain::profile::Profile;

/// Longest message kept in one log line. Longer text is cut.
const MAX_MESSAGE_CHARS: usize = 500;
/// Longest error kind kept in one log line. Longer text is cut.
const MAX_KIND_CHARS: usize = 40;

/// Replaces every control character (line breaks, tabs, escapes, NUL) with a space,
/// so one event stays one printable log line.
fn one_line(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// The app.log line for a webview error. Kind and message are cut and made one line.
pub fn webview_log_line(kind: &str, message: &str) -> String {
    let kind: String = one_line(kind).chars().take(MAX_KIND_CHARS).collect();
    let message: String = one_line(message).chars().take(MAX_MESSAGE_CHARS).collect();
    format!("editor webview: {kind}: {message}")
}

/// The app.log line for a save refused by the layout rules. Logs the count only: the
/// problem text can carry action payloads (commands, URLs, hotkeys) and must stay out.
pub fn save_refused_line(problem_count: usize) -> String {
    format!("editor: save refused: {problem_count} problem(s), details not logged")
}

/// The app.log line for a saved layout: its key count and file path.
pub fn saved_line(profile: &Profile, path: &str) -> String {
    let keys: usize = profile.pages.iter().map(|page| page.keys.len()).sum();
    format!("editor: saved profile ({keys} keys) to {}", one_line(path))
}

/// The app.log line for a failed layout save.
pub fn save_failed_line(err: &str) -> String {
    format!("editor: save failed: {}", one_line(err))
}

/// Logs an error the editor webview caught. Fire-and-forget from the frontend.
#[tauri::command]
#[specta::specta]
pub fn log_webview_error(kind: String, message: String) {
    tracing::warn!("{}", webview_log_line(&kind, &message));
}

#[cfg(test)]
#[path = "editor_log_tests.rs"]
mod tests;
