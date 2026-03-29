mod application;
mod domain;
mod infrastructure;
mod logging;
mod presentation;

use application::{handle_filter_query, handle_key_event, render_screen};
use domain::navigation::NavigationStack;
use infrastructure::persistence::DeviceState;
use infrastructure::usb::{
    clear_all, device_init, keep_alive, read_event, reset_endpoints, set_brightness, PID, VID,
};
use rusb::{Context, UsbContext as _};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Arc::new(Mutex::new(presentation::tui::AppState::new(2)));
    let _log_guard = logging::init(Arc::clone(&app_state));

    let ctx = Context::new().map_err(|e| format!("rusb context: {e}"))?;
    let handle = ctx
        .open_device_with_vid_pid(VID, PID)
        .ok_or("Device not found — is it plugged in?")?;
    handle.detach_kernel_driver(0).ok(); // terminates AppleUserHIDDrivers dext
    handle
        .claim_interface(0)
        .map_err(|e| format!("Failed to claim USB interface — run 'make install' once to set up permissions, or use sudo: {e}"))?;
    info!("interface 0 claimed");
    reset_endpoints(&handle);

    let dev_state_val = DeviceState::load();

    device_init(&handle);
    clear_all(&handle);
    set_brightness(&handle, dev_state_val.brightness);
    info!("device ready");

    let shutdown = Arc::new(AtomicBool::new(false));
    let dev_state = Arc::new(Mutex::new(dev_state_val));

    {
        let s = Arc::clone(&app_state);
        let q = Arc::clone(&shutdown);
        std::thread::spawn(move || presentation::tui::run(s, q));
    }

    let initial_page = dev_state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .current_page;
    let mut nav = NavigationStack::new(initial_page, 2);
    render_screen(&nav, &handle, &app_state, &dev_state);
    info!("listening — press keys (11=back, 12=out, 13=fwd)");

    let mut last_heartbeat = Instant::now();

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        let confirmed_query = {
            let mut s = app_state.lock().unwrap_or_else(|e| e.into_inner());
            if s.text_input_confirmed {
                s.text_input_confirmed = false;
                Some(s.text_input_value.clone())
            } else {
                None
            }
        };
        if let Some(query) = confirmed_query {
            handle_filter_query(&query, &handle, &mut nav, &app_state, &dev_state);
        }

        if let Some(data) = read_event(&handle, Duration::from_millis(500)) {
            if data.len() >= 11 {
                handle_key_event(&data, &handle, &mut nav, &app_state, &dev_state);
            }
        }

        if last_heartbeat.elapsed() >= Duration::from_secs(8) {
            keep_alive(&handle);
            last_heartbeat = Instant::now();
        }
    }

    Ok(())
}
