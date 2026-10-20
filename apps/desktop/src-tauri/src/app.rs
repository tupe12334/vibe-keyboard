//! App state shared by the commands, the tray and the controller thread.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::SystemTime;

use tauri::AppHandle;
use vibe_keyboard::controller::{self, RunOptions};
use vibe_keyboard::infrastructure::profile_store;
use vibe_keyboard::presentation::tui::AppState;

struct Running {
    shutdown: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

/// Managed by Tauri. One controller thread at most.
pub struct App {
    /// UI state shared with the logger. The controller loads the profile into it.
    state: Arc<Mutex<AppState>>,
    running: Mutex<Option<Running>>,
    pub(crate) device_connected: AtomicBool,
    /// Profile file mtime when the editor last loaded or saved it. Save refuses when it differs.
    pub loaded_mtime: Mutex<Option<SystemTime>>,
}

impl App {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self {
            state,
            running: Mutex::new(None),
            device_connected: AtomicBool::new(false),
            loaded_mtime: Mutex::new(None),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }

    pub fn is_device_connected(&self) -> bool {
        self.device_connected.load(Ordering::Relaxed)
    }

    /// Start the controller on a background thread. Does nothing when it already runs.
    /// `VIBE_NO_DEVICE` set: UI only, the controller never starts and never opens the deck.
    pub fn start(&self, app: &AppHandle) {
        if std::env::var_os("VIBE_NO_DEVICE").is_some() {
            return;
        }
        let mut slot = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        if slot.is_some() {
            return;
        }
        let shutdown = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&shutdown);
        let handle = app.clone();
        let state = Arc::clone(&self.state);
        let thread = std::thread::spawn(move || {
            let options = RunOptions {
                headless: true,
                shutdown: flag,
                state,
                on_event: Box::new(move |event| crate::events::on_event(&handle, event)),
            };
            if let Err(e) = controller::run(options) {
                eprintln!("controller stopped: {e}");
            }
        });
        *slot = Some(Running { shutdown, thread });
    }

    /// Stop the controller and wait for it. The controller clears the deck on the way out.
    pub fn stop(&self) {
        let running = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(running) = running {
            running.shutdown.store(true, Ordering::Relaxed);
            let _ = running.thread.join();
        }
    }

    pub fn record_profile_mtime(&self) {
        *self
            .loaded_mtime
            .lock()
            .unwrap_or_else(PoisonError::into_inner) =
            profile_store::modified_at(&profile_store::profile_path());
    }
}
