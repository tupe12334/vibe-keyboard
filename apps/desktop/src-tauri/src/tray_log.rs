//! Tray "Show log": opens `app.log` in Console, or in the default app when Console fails.
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use tauri::menu::MenuItem;
use tauri::{AppHandle, Wry};
use tracing::{error, warn};

/// Tray item id for "Show log".
pub const MENU_ID: &str = "show_log";

/// The "Show log" tray item.
pub fn menu_item(app: &AppHandle) -> tauri::Result<MenuItem<Wry>> {
    MenuItem::with_id(app, MENU_ID, "Show log", true, None::<&str>)
}

/// True when a tray menu id is the "Show log" item.
pub fn is_show_log(id: &str) -> bool {
    id == MENU_ID
}

/// `open` arguments that open `path` in Console.
pub fn console_args(path: &Path) -> [&OsStr; 3] {
    [OsStr::new("-a"), OsStr::new("Console"), path.as_os_str()]
}

/// `open` arguments that open `path` in its default app.
pub fn plain_args(path: &Path) -> [&OsStr; 1] {
    [path.as_os_str()]
}

/// Open the log. A missing file logs a warning and does nothing.
pub fn show_log() {
    let path = vibe_keyboard::logging::log_file();
    if !path.exists() {
        warn!("app.log not found at {}; nothing to show", path.display());
        return;
    }
    if !run_open(&console_args(&path)) && !run_open(&plain_args(&path)) {
        error!("could not open {}", path.display());
    }
}

/// Run `open` and wait for it. True when it exits 0.
fn run_open(args: &[&OsStr]) -> bool {
    Command::new("open")
        .args(args)
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(test)]
#[path = "tray_log_tests.rs"]
mod tests;
