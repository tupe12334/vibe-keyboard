#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

include!("modules.rs");

use std::sync::{Arc, Mutex};

use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use vibe_keyboard::infrastructure::login_path::apply_login_path;
use vibe_keyboard::presentation::tui::AppState;

use app::App;

fn main() {
    // First, before any thread or child process. A login start has a minimal PATH.
    let login_path_dirs = apply_login_path();
    // Held for the whole run: dropping it stops the file log.
    let state = Arc::new(Mutex::new(AppState::new(0)));
    let _log_guard = vibe_keyboard::logging::init(Arc::clone(&state));
    tracing::info!("PATH extended at login: {login_path_dirs:?}");

    let builder = bindings::specta_builder();

    #[cfg(debug_assertions)]
    bindings::export_bindings(&builder);

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(App::new(state))
        .manage(quit::QuitGuard::default())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            tray::setup_tray(app.handle())?;
            // A refusal (another controller holds the deck) shows in the editor's device status.
            let _ = app.state::<App>().start(app.handle());
            Ok(())
        })
        // Closing the window hides it. The controller keeps running until Quit.
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building Vibe Keyboard")
        .run(|app, event| match event {
            RunEvent::Ready => editor_launch::open_at_launch(app),
            RunEvent::ExitRequested { api, .. } => quit::on_exit_requested(app, &api),
            _ => {}
        });
}
