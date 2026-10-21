//! Editor events for app.log. The editor console is hidden, so its errors and saves are logged here.

use vibe_keyboard::domain::profile::Profile;

/// Longest message kept in one log line. Longer text is cut.
const MAX_MESSAGE_CHARS: usize = 500;

/// Replaces line breaks with spaces, so one event stays one log line.
fn one_line(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect()
}

/// The app.log line for a webview error. Cut to 500 characters, no line breaks.
pub fn webview_log_line(kind: &str, message: &str) -> String {
    let message: String = one_line(message).chars().take(MAX_MESSAGE_CHARS).collect();
    format!("editor webview: {}: {}", one_line(kind), message)
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
