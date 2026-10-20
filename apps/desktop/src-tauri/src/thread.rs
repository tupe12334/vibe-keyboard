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
use crate::tray;

/// Run the controller on a new thread. `lock` is held until the thread ends.
pub fn spawn(
    app: AppHandle,
    state: Arc<Mutex<AppState>>,
    shutdown: Arc<AtomicBool>,
    lock: ControllerLock,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let _lock = lock;
        let mut report = ExitReport {
            app: app.clone(),
            failure: None,
        };
        let on_app = app.clone();
        let options = RunOptions {
            headless: true,
            shutdown,
            state,
            on_event: Box::new(move |event| events::on_event(&on_app, event)),
        };
        if let Err(e) = controller::run(options) {
            report.failure = Some(format!("controller stopped: {e}"));
        }
    })
}

/// Reports the thread's exit when it is dropped, including a panic unwinding through it.
struct ExitReport {
    app: AppHandle,
    failure: Option<String>,
}

impl Drop for ExitReport {
    fn drop(&mut self) {
        let reason = if std::thread::panicking() {
            Some("controller stopped unexpectedly. See app.log.".to_string())
        } else {
            self.failure.take()
        };
        let state = self.app.state::<App>();
        state
            .device_connected
            .store(false, std::sync::atomic::Ordering::Relaxed);
        state.set_error(reason);
        tray::set_controller_running(&self.app, false);
        let _ = DeviceDisconnected {}.emit(&self.app);
    }
}
