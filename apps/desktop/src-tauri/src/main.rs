#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod accessibility;
mod app;
mod app_commands;
mod bindings;
#[cfg(test)]
mod command_harness;
mod commands;
mod display_commands;
#[cfg(test)]
mod display_tests;
#[cfg(test)]
mod editor_preserve_hold_tests;
#[cfg(test)]
mod editor_preserve_key_tests;
#[cfg(test)]
mod editor_preserve_layout_tests;
#[cfg(test)]
mod editor_preserve_side_tests;
#[cfg(test)]
mod editor_preserve_startup_tests;
#[cfg(test)]
mod editor_preserve_support;
mod events;
mod key_commands;
mod layout_commands;
mod lifecycle;
mod login_item;
mod photo_commands;
mod profile_io;
#[cfg(test)]
mod profile_roundtrip_tests;
mod prompt_commands;
mod quit;
mod side_commands;
#[cfg(test)]
mod side_tests;
mod status;
mod supervise;
mod thread;
mod tray;

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
        .run(|app, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                quit::on_exit_requested(app, &api);
            }
        });
}
