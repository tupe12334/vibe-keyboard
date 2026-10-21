//! Controller run: set up the deck, then drive the device loop until it stops.
use super::ctx::{Ctx, Flow};
use super::event::ControllerEvent;
use super::lost::recover;
use super::setup::open_device;
use super::stop::stop_if_none;
use super::tick;
use crate::application::{release_all_holds, render_screen};
use crate::domain::{navigation::NavigationStack, ptt::Release};
use crate::infrastructure::profile_store;
use crate::infrastructure::usb::{clear_all, DeckPort, Hardware};
use crate::presentation;
use std::sync::{Arc, Mutex};
use tracing::info;

pub use super::options::{EventSink, RunOptions};

type RunResult = Result<(), Box<dyn std::error::Error>>;

/// Run the device loop. Call only while holding the `claim()` lock.
pub fn run(opts: RunOptions) -> RunResult {
    run_with(opts, &Hardware)
}

/// `run` on a given deck. Tests pass a fake `DeckPort`.
pub(super) fn run_with(opts: RunOptions, port: &dyn DeckPort) -> RunResult {
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
    let Some((mut handle, dev_state_val)) = stop_if_none(open_device(port, &shutdown), &*on_event)
    else {
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
        std::thread::spawn(move || presentation::tui::run(&s, &q));
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
                release_all_holds(&ctx.app_state, Release::Shutdown);
                clear_all(&handle);
                (ctx.events)(ControllerEvent::Stopped);
                return Ok(());
            }
            Flow::Lost => {
                let Some(new_handle) = recover(port, handle, &mut ctx) else {
                    return Ok(());
                };
                handle = new_handle;
            }
        }
    }
}
