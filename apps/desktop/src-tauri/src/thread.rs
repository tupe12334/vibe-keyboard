//! The controller thread. Its exit reaches the tray and the webview, panic or not.
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use vibe_keyboard::controller::{self, RunOptions};
use vibe_keyboard::infrastructure::controller_lock::ControllerLock;
use vibe_keyboard::presentation::tui::AppState;

use crate::app::App;
use crate::events::{self, DeviceDisconnected};
use crate::supervise::spawn_controller;
use crate::tray;

/// Run the controller on a new thread. `lock` is held until the body ends.
pub fn spawn(
    app: AppHandle,
    state: Arc<Mutex<AppState>>,
    shutdown: Arc<AtomicBool>,
    lock: ControllerLock,
) -> JoinHandle<()> {
    let exit_app = app.clone();
    spawn_controller(
        move || {
            let _lock = lock;
            let on_app = app.clone();
            let options = RunOptions {
                headless: true,
                shutdown,
                state,
                on_event: Box::new(move |event| events::on_event(&on_app, event)),
            };
            controller::run(options)
                .err()
                .map(|e| format!("controller stopped: {e}"))
        },
        move |reason| report_exit(&exit_app, reason),
    )
}

/// Record the exit, then update the tray and the webview on the main thread. Never waits for
/// it: Pause and Quit join this thread from the main thread.
fn report_exit(app: &AppHandle, reason: Option<String>) {
    let state = app.state::<App>();
    state
        .device_connected
        .store(false, std::sync::atomic::Ordering::Relaxed);
    state.set_error(reason);
    let main = app.clone();
    let _ = app.run_on_main_thread(move || {
        tray::set_controller_running(&main, false);
        let _ = DeviceDisconnected {}.emit(&main);
    });
}
