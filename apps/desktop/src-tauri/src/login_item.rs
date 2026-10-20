//! Start at login: the tray's checkbox.
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::tray::TrayItems;

pub fn toggle_autostart(app: &AppHandle) {
    let manager = app.autolaunch();
    let enable = !manager.is_enabled().unwrap_or(false);
    let result = if enable {
        manager.enable()
    } else {
        manager.disable()
    };
    if result.is_ok() {
        let _ = app.state::<TrayItems>().autostart.set_checked(enable);
    }
}
