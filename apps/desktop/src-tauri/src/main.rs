#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod bindings;
#[cfg(test)]
mod command_harness;
mod commands;
mod events;
mod key_commands;
mod layout_commands;
mod lifecycle;
mod login_item;
mod profile_io;
mod status;
mod supervise;
mod thread;
mod tray;

use std::sync::{Arc, Mutex};

use tauri::{Manager, WindowEvent, Wry};
use tauri_plugin_autostart::MacosLauncher;
use tauri_specta::{collect_commands, collect_events, Builder};
use vibe_keyboard::presentation::tui::AppState;

use app::App;

/// The command and event list the frontend binds to. Also the source of the TypeScript bindings.
fn specta_builder() -> Builder<Wry> {
    Builder::<Wry>::new()
        .commands(collect_commands![
            commands::get_profile,
            commands::save_profile,
            key_commands::key_preview,
            key_commands::list_builtins,
            key_commands::action_problem,
            commands::device_status,
            layout_commands::editor_rules,
            layout_commands::add_page,
            layout_commands::delete_page,
        ])
        .events(collect_events![
            events::DeviceConnected,
            events::DeviceDisconnected,
            events::KeyPressed,
            events::ProfileReloaded,
        ])
}

fn main() {
    // Held for the whole run: dropping it stops the file log.
    let state = Arc::new(Mutex::new(AppState::new(0)));
    let _log_guard = vibe_keyboard::logging::init(Arc::clone(&state));

    let builder = specta_builder();

    #[cfg(debug_assertions)]
    bindings::export_bindings(&builder);

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(App::new(state))
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
        .run(tauri::generate_context!())
        .expect("error while running Vibe Keyboard");
}
