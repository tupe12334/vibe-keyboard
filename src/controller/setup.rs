//! Startup: load the profile, open the deck, and reconnect when it returns.
use super::ctx::Ctx;
use super::reload::log_problems;
use crate::application::render_screen;
use crate::domain::profile::Profile;
use crate::infrastructure::mouse_release::clear_stuck_left_button;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::profile_store;
use crate::infrastructure::usb::{device_init, wait_for_device, Device};
use crate::presentation::tui::AppState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

/// Load the profile file into `state`. Call after logging is set up.
pub fn load_profile(state: &Arc<Mutex<AppState>>) {
    load_profile_at(state, &profile_store::profile_path());
}

/// `load_profile` from a given file. Writes the default profile only when the file is missing.
pub fn load_profile_at(state: &Arc<Mutex<AppState>>, path: &std::path::Path) {
    // Startup is the only place that may write the default profile.
    let (profile, profile_error) = match profile_store::ensure_default(path) {
        Ok(profile) => (profile, None),
        Err(e) => (Profile::default_from_legacy(), Some(e)),
    };
    let page_count = profile.pages.len();
    if let Some(e) = profile_error {
        error!("profile: {e}; using built-in layout");
    }
    log_problems(&profile);
    let mut s = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    s.total_pages = page_count;
    s.profile = profile;
}

/// Open the deck, waiting for it if needed. None when shutdown comes first.
pub fn open_device(shutdown: &AtomicBool) -> Option<(Device, DeviceState)> {
    // Stop first: a shutdown already requested must not probe the deck.
    if shutdown.load(Ordering::Relaxed) {
        return None;
    }
    let state = DeviceState::load();
    let brightness = state.brightness;
    let dev = match Device::open() {
        Ok(dev) => {
            device_init(&dev, brightness);
            dev
        }
        Err(e) => {
            info!("waiting for device: {e}");
            wait_for_device(shutdown, brightness)?
        }
    };
    clear_stuck_left_button();
    Some((dev, state))
}

/// Drop the lost handle, wait for the deck, and redraw. None when shutdown comes first.
pub fn reconnect(handle: Device, c: &Ctx) -> Option<Device> {
    // Release the stale handle before waiting for the device to come back.
    drop(handle);
    let brightness = c
        .dev_state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .brightness;
    let new_handle = wait_for_device(&c.shutdown, brightness)?;
    // Re-render the current screen from the same page images the editor previews.
    render_screen(&c.nav, &new_handle, &c.app_state, &c.dev_state);
    info!("device reconnected");
    clear_stuck_left_button();
    Some(new_handle)
}

#[cfg(test)]
#[path = "setup_tests.rs"]
mod tests;
