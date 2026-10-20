//! App state shared by the commands, the tray and the controller thread.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use tauri::AppHandle;
use vibe_keyboard::controller;
use vibe_keyboard::infrastructure::controller_lock::{self, AcquireError};
use vibe_keyboard::infrastructure::profile_save::Version;
use vibe_keyboard::presentation::tui::AppState;

use crate::lifecycle::{start_with, stop_running, Running};
use crate::thread::spawn;

/// Managed by Tauri. One controller thread at most.
pub struct App {
    /// UI state shared with the logger. The controller loads the profile into it.
    pub(crate) state: Arc<Mutex<AppState>>,
    pub(crate) running: Mutex<Option<Running>>,
    pub(crate) device_connected: AtomicBool,
    /// Why the controller refused to start or stopped with an error. None when fine.
    pub(crate) error: Mutex<Option<String>>,
    /// Profile file version (content hash) when the editor last loaded or saved it. Save refuses when it differs.
    pub loaded_version: Mutex<Option<Version>>,
}

impl App {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self {
            state,
            running: Mutex::new(None),
            device_connected: AtomicBool::new(false),
            error: Mutex::new(None),
            loaded_version: Mutex::new(None),
        }
    }

    /// Start the controller on a background thread. Does nothing when it already runs.
    /// Refuses (and says why) when another controller holds the deck.
    /// `VIBE_NO_DEVICE` set: UI only, the controller never starts and never opens the deck.
    pub fn start(&self, app: &AppHandle) -> Result<(), String> {
        if std::env::var_os("VIBE_NO_DEVICE").is_some() {
            return Ok(());
        }
        start_with(&self.running, || {
            let lock = match controller::claim() {
                Ok(lock) => lock,
                Err(AcquireError::Held) => {
                    let message = controller::held_message(&controller_lock::default_lock_path());
                    self.set_error(Some(message.clone()));
                    return Err(message);
                }
                Err(AcquireError::Io(e)) => {
                    let message = format!("controller lock: {e}");
                    self.set_error(Some(message.clone()));
                    return Err(message);
                }
            };
            // The same profile load as `listen`. The controller needs the page count.
            controller::load_profile(&self.state);
            self.set_error(None);
            self.check_accessibility(app);
            let shutdown = Arc::new(AtomicBool::new(false));
            let thread = spawn(
                app.clone(),
                Arc::clone(&self.state),
                Arc::clone(&shutdown),
                lock,
            );
            Ok(Running { shutdown, thread })
        })
    }

    /// Stop the controller and wait for it. The controller clears the deck on the way out.
    pub fn stop(&self) {
        stop_running(&self.running);
    }
}
