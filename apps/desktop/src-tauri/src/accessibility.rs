//! Accessibility check for push-to-talk. The tray item and the editor's status show the warning.
use std::sync::PoisonError;

use tauri::menu::MenuItem;
use tauri::{AppHandle, Manager, Wry};
use tracing::{error, warn};
use vibe_keyboard::domain::accessibility::{
    push_to_talk_warning, uses_push_to_talk, PUSH_TO_TALK_WARNING,
};
use vibe_keyboard::infrastructure::keyboard::accessibility_trusted;

use crate::app::App;
use crate::tray::TrayItems;

/// Tray item id. Clicking it opens the Accessibility settings.
pub const MENU_ID: &str = "accessibility";

/// The Privacy > Accessibility pane of System Settings.
const SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

impl App {
    /// Check the layout on the deck. Shows the warning in the editor's status and the tray,
    /// or clears it. The OS and the layout are read here; the decision is in the core crate.
    pub(crate) fn check_accessibility(&self, app: &AppHandle) {
        let uses = uses_push_to_talk(
            &self
                .state
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .profile,
        );
        let warning = push_to_talk_warning(accessibility_trusted(), uses);
        if let Some(text) = warning {
            warn!("{text}");
        }
        self.set_error(warning.map(str::to_string));
        let show = warning.is_some();
        let main = app.clone();
        let _ = app.run_on_main_thread(move || set_menu_warning(&main, show));
    }
}

/// Open the Accessibility pane from the tray.
#[allow(
    clippy::zombie_processes,
    reason = "the settings opener runs detached and is never waited on"
)]
pub fn open_settings() {
    if let Err(e) = std::process::Command::new("open").arg(SETTINGS_URL).spawn() {
        error!("could not open Accessibility settings: {e}");
    }
}

/// The tray item that shows the warning. Built once in the tray; shown only while needed.
pub fn menu_item(app: &AppHandle) -> tauri::Result<MenuItem<Wry>> {
    MenuItem::with_id(app, MENU_ID, PUSH_TO_TALK_WARNING, true, None::<&str>)
}

/// Show or hide the tray item at the top of the menu.
fn set_menu_warning(app: &AppHandle, show: bool) {
    let Some(items) = app.try_state::<TrayItems>() else {
        return;
    };
    let shown = items
        .menu
        .items()
        .unwrap_or_default()
        .iter()
        .any(|item| item.id().as_ref() == MENU_ID);
    if show && !shown {
        let _ = items.menu.insert(&items.accessibility, 0);
    } else if !show && shown {
        let _ = items.menu.remove(&items.accessibility);
    }
}
