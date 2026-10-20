//! Quit asks the editor first, so a quit never drops unsaved layout edits.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::app::App;
use crate::events::QuitRequested;

/// A quit repeated within this window exits when the editor has not answered the first one.
const REPEAT_WINDOW: Duration = Duration::from_secs(5);

/// Managed by Tauri. Set when the editor confirms the quit. The exit that follows skips the ask.
#[derive(Default)]
pub struct QuitGuard {
    confirmed: AtomicBool,
    /// When the last quit request was asked and the editor has not acknowledged it yet.
    unanswered_since: Mutex<Option<Instant>>,
}

/// True when an unanswered quit request was asked within the repeat window before `now`.
pub fn is_repeat(unanswered_since: Option<Instant>, now: Instant) -> bool {
    unanswered_since.is_some_and(|since| now.duration_since(since) <= REPEAT_WINDOW)
}

/// An exit waits for the editor only when the editor is open, has loaded the layout, and has not
/// confirmed yet. Before the first load no edit can exist, so nothing unsaved can be lost.
pub fn must_ask(confirmed: bool, has_editor: bool, editor_loaded: bool) -> bool {
    !confirmed && has_editor && editor_loaded
}

/// Runs on every `ExitRequested`: tray Quit, Cmd+Q, or `app.exit`.
pub fn on_exit_requested(app: &AppHandle, api: &tauri::ExitRequestApi) {
    let guard = app.state::<QuitGuard>();
    let confirmed = guard.confirmed.load(Ordering::SeqCst);
    let has_editor = app.get_webview_window("main").is_some();
    let editor_loaded = app.state::<App>().editor_loaded.load(Ordering::SeqCst);
    if must_ask(confirmed, has_editor, editor_loaded) {
        let now = Instant::now();
        let mut unanswered = guard
            .unanswered_since
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // A second quit with no answer to the first: the editor is dead, so this one goes ahead.
        if !is_repeat(*unanswered, now) {
            *unanswered = Some(now);
            drop(unanswered);
            api.prevent_exit();
            crate::tray::show_editor(app);
            // Emit errors only mean no webview is listening. The editor is open here, so none is.
            let _ = QuitRequested {}.emit(app);
            return;
        }
    }
    // The exit goes ahead: stop the controller so it clears the deck on the way out.
    app.state::<App>().stop();
}

/// The editor got a quit request and is alive. The next quit asks again.
#[tauri::command]
#[specta::specta]
pub fn acknowledge_quit(guard: State<'_, QuitGuard>) {
    *guard
        .unanswered_since
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = None;
}

/// The editor allows the quit: nothing unsaved, or Save or Discard was chosen.
#[tauri::command]
#[specta::specta]
pub fn confirm_quit(app: AppHandle, guard: State<'_, QuitGuard>) {
    guard.confirmed.store(true, Ordering::SeqCst);
    app.exit(0);
}

#[cfg(test)]
#[path = "quit_tests.rs"]
mod tests;
