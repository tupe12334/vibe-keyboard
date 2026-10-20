//! One pass of the device loop: timers and app events. Key input is in `input.rs`.
use super::ctx::{Ctx, Flow};
use super::event::ControllerEvent;
use super::input;
use super::reload::reload_profile;
use super::side;
use crate::application::{handle_filter_query, handle_frontmost, prompt_return, render_screen};
use crate::infrastructure::images::app_icon_generation;
use crate::infrastructure::profile_store;
use crate::infrastructure::usb::Device;
use std::sync::atomic::Ordering;
use std::sync::PoisonError;
use std::time::{Duration, Instant};

const PROFILE_POLL: Duration = Duration::from_secs(1);

pub fn step(c: &mut Ctx, handle: &Device) -> Flow {
    if c.shutdown.load(Ordering::Relaxed) {
        return Flow::Stop;
    }
    poll_profile(c, handle);
    while let Ok(bundle_id) = c.frontmost_rx.try_recv() {
        handle_frontmost(&bundle_id, &mut c.nav, handle, &c.app_state, &c.dev_state);
    }
    redraw_on_icon_change(c, handle);
    confirm_filter(c, handle);
    let read_lost = input::read_key(c, handle);
    side::tick(c, handle);
    input::heartbeat(c, handle);
    input::back_on_main_page(c, handle);
    c.pages.tick(&mut c.nav, handle, &c.app_state, &c.dev_state);
    prompt_return::poll_sent_prompt(&mut c.nav, handle, &c.app_state, &c.dev_state);
    if read_lost || handle.has_failed() {
        Flow::Lost
    } else {
        Flow::Continue
    }
}

/// Reload the profile when its file changed on disk.
fn poll_profile(c: &mut Ctx, handle: &Device) {
    if c.last_profile_check.elapsed() < PROFILE_POLL {
        return;
    }
    c.last_profile_check = Instant::now();
    let mtime = profile_store::modified_at(&profile_store::profile_path());
    if mtime != c.last_profile_mtime {
        c.last_profile_mtime = mtime;
        if reload_profile(&c.app_state, &mut c.nav, handle, &c.dev_state) {
            (c.events)(ControllerEvent::ProfileReloaded);
        }
    }
}

/// A background app-icon lookup finished: redraw so the key shows its icon.
fn redraw_on_icon_change(c: &mut Ctx, handle: &Device) {
    let generation = app_icon_generation();
    if generation != c.icon_generation {
        c.icon_generation = generation;
        if c.nav.main_page().is_some() {
            render_screen(&c.nav, handle, &c.app_state, &c.dev_state);
        }
    }
}

fn confirm_filter(c: &mut Ctx, handle: &Device) {
    let confirmed = {
        let mut s = c.app_state.lock().unwrap_or_else(PoisonError::into_inner);
        if s.text_input_confirmed {
            s.text_input_confirmed = false;
            Some(s.text_input_value.clone())
        } else {
            None
        }
    };
    if let Some(query) = confirmed {
        handle_filter_query(&query, handle, &mut c.nav, &c.app_state, &c.dev_state);
    }
}
