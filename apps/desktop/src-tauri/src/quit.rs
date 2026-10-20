//! Quit asks the editor first, so a quit never drops unsaved layout edits.
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::app::App;
use crate::events::QuitRequested;

/// Managed by Tauri. Set when the editor confirms the quit. The exit that follows skips the ask.
#[derive(Default)]
pub struct QuitGuard {
    confirmed: AtomicBool,
}

/// An exit waits for the editor only when the editor is open, has loaded the layout, and has not
/// confirmed yet. Before the first load no edit can exist, so nothing unsaved can be lost.
pub fn must_ask(confirmed: bool, has_editor: bool, editor_loaded: bool) -> bool {
    !confirmed && has_editor && editor_loaded
}

/// Runs on every `ExitRequested`: tray Quit, Cmd+Q, or `app.exit`.
pub fn on_exit_requested(app: &AppHandle, api: &tauri::ExitRequestApi) {
    let confirmed = app.state::<QuitGuard>().confirmed.load(Ordering::SeqCst);
    let has_editor = app.get_webview_window("main").is_some();
    let editor_loaded = app.state::<App>().editor_loaded.load(Ordering::SeqCst);
    if must_ask(confirmed, has_editor, editor_loaded) {
        api.prevent_exit();
        crate::tray::show_editor(app);
        // Emit errors only mean no webview is listening. The editor is open here, so none is.
        let _ = QuitRequested {}.emit(app);
    } else {
        // The exit goes ahead: stop the controller so it clears the deck on the way out.
        app.state::<App>().stop();
    }
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
