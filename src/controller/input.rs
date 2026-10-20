//! Key input, keep-alive, and app switches. Called from each device-loop pass.
use super::ctx::Ctx;
use super::event::ControllerEvent;
use super::keys::key_press;
use crate::application::{handle_frontmost, handle_key_event, read_wait};
use crate::infrastructure::frontmost;
use crate::infrastructure::usb::{keep_alive, read_event, Device};
use std::time::{Duration, Instant};
use tracing::warn;

/// Read one key event. Returns true when the read failed, which means the device is lost.
pub fn read_key(c: &mut Ctx, handle: &Device) -> bool {
    match read_event(handle, read_wait(&c.app_state)) {
        Ok(Some(data)) => {
            if data.len() >= 11 {
                handle_key_event(
                    &data,
                    handle,
                    &mut c.nav,
                    &c.app_state,
                    &c.dev_state,
                    &mut c.pages,
                );
                if let Some(key) = key_press(&data) {
                    (c.events)(ControllerEvent::KeyPressed(key));
                }
            }
            false
        }
        Ok(None) => false,
        Err(e) => {
            warn!("USB read error: {e}");
            true
        }
    }
}

pub fn heartbeat(c: &mut Ctx, handle: &Device) {
    if c.last_heartbeat.elapsed() >= Duration::from_secs(8) {
        keep_alive(handle);
        c.last_heartbeat = Instant::now();
    }
}

/// Back on a main page after a sub-screen: the app in front may have changed meanwhile.
pub fn back_on_main_page(c: &mut Ctx, handle: &Device) {
    let on_main_page = c.nav.main_page().is_some();
    if on_main_page && !c.was_on_main_page {
        if let Some(bundle_id) = frontmost::frontmost_bundle_id() {
            handle_frontmost(&bundle_id, &mut c.nav, handle, &c.app_state, &c.dev_state);
        }
    }
    c.was_on_main_page = on_main_page;
}
