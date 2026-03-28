mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::{activate_page, handle_key_event};
use domain::navigation::Navigator;
use infrastructure::persistence::DeviceState;
use infrastructure::usb::{clear_all, device_init, keep_alive, read_event, reset_endpoints, set_brightness, PID, VID};
use rusb::{Context, UsbContext as _};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn main() {
    let ctx = Context::new().expect("rusb context");
    let handle = ctx
        .open_device_with_vid_pid(VID, PID)
        .expect("Device not found — is it plugged in? Try: sudo cargo run");
    handle.detach_kernel_driver(0).ok(); // terminates AppleUserHIDDrivers dext
    handle
        .claim_interface(0)
        .expect("Failed to claim USB interface 0 — try: sudo cargo run");
    println!("[init] interface 0 claimed");
    reset_endpoints(&handle);

    let dev_state_val = DeviceState::load();

    device_init(&handle);
    clear_all(&handle);
    set_brightness(&handle, dev_state_val.brightness);
    println!("[init] device ready");

    let app_state = Arc::new(Mutex::new(presentation::tui::AppState::new(2)));
    let shutdown  = Arc::new(AtomicBool::new(false));
    let dev_state = Arc::new(Mutex::new(dev_state_val));

    {
        let s = Arc::clone(&app_state);
        let q = Arc::clone(&shutdown);
        std::thread::spawn(move || presentation::tui::run(s, q));
    }

    let initial_page = dev_state.lock().unwrap().current_page;
    let mut nav = Navigator::new(2);
    nav.go(initial_page);
    activate_page(nav.current(), &handle, &app_state, &dev_state);
    println!("[init] listening — press keys (11=back, 12=forward)");

    let mut last_heartbeat = Instant::now();

    loop {
        if shutdown.load(Ordering::Relaxed) { break; }

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
}
