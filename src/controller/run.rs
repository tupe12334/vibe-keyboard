//! Controller run: set up the deck, then drive the device loop until it stops.
use super::ctx::{Ctx, Flow};
use super::event::ControllerEvent;
use super::setup::{open_device, reconnect};
use super::stop::stop_if_none;
use super::tick;
use crate::application::render_screen;
use crate::domain::navigation::NavigationStack;
use crate::infrastructure::profile_store;
use crate::infrastructure::usb::clear_all;
use crate::presentation;
use crate::presentation::tui::AppState;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{info, warn};

/// Called on the controller thread for each event.
pub type EventSink = Box<dyn Fn(ControllerEvent) + Send>;

pub struct RunOptions {
    /// Skip the ratatui TUI. No terminal is needed.
    pub headless: bool,
    /// Set this to stop the loop. Signal handlers set it too.
    pub shutdown: Arc<AtomicBool>,
    /// UI state. The caller starts logging with it and loads the profile (`load_profile`).
    pub state: Arc<Mutex<AppState>>,
    pub on_event: EventSink,
}

/// Run the device loop. Call only while holding the `claim()` lock.
pub fn run(opts: RunOptions) -> Result<(), Box<dyn std::error::Error>> {
    let RunOptions {
        headless,
        shutdown,
        state: app_state,
        on_event,
    } = opts;
    let page_count = app_state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .total_pages;
    // Snapshot before the device wait, so a profile edit made while unplugged reloads.
    let last_profile_mtime = profile_store::modified_at(&profile_store::profile_path());
    let Some((mut handle, dev_state_val)) = stop_if_none(open_device(&shutdown), &*on_event) else {
        return Ok(());
    };
    info!("device ready");
    on_event(ControllerEvent::DeviceReady);

    let dev_state = Arc::new(Mutex::new(dev_state_val));
    if headless {
        info!("headless mode: no TUI");
    } else {
        let s = Arc::clone(&app_state);
        let q = Arc::clone(&shutdown);
        std::thread::spawn(move || presentation::tui::run(s, q));
    }

    let initial_page = dev_state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .current_page;
    let nav = NavigationStack::new(initial_page, page_count);
    render_screen(&nav, &handle, &app_state, &dev_state);
    info!("listening — press keys (11=back, 12=out, 13=fwd)");

    let mut ctx = Ctx::new(
        app_state,
        dev_state,
        nav,
        shutdown,
        on_event,
        last_profile_mtime,
    );

    loop {
        match tick::step(&mut ctx, &handle) {
            Flow::Continue => {}
            Flow::Stop => {
                info!("stopping");
                clear_all(&handle);
                (ctx.events)(ControllerEvent::Stopped);
                return Ok(());
            }
            Flow::Lost => {
                warn!("device disconnected");
                (ctx.events)(ControllerEvent::DeviceLost);
                let Some(new_handle) = stop_if_none(reconnect(handle, &ctx), &*ctx.events) else {
                    return Ok(());
                };
                handle = new_handle;
                (ctx.events)(ControllerEvent::DeviceRestored);
                ctx.last_heartbeat = Instant::now();
            }
        }
    }
}
