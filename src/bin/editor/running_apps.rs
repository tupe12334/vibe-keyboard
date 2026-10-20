use std::sync::mpsc;

use vibe_keyboard::infrastructure::running_apps::running_apps;

impl super::Editor {
    /// Run `lsappinfo` on a background thread. `ui` picks up the result.
    pub(super) fn start_running_refresh(&mut self) {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(running_apps());
        });
        self.running_rx = Some(rx);
    }

    pub(super) fn poll_running(&mut self) {
        if let Some(rx) = &self.running_rx {
            if let Ok(apps) = rx.try_recv() {
                self.running = apps;
                self.running_rx = None;
            }
        }
    }
}
