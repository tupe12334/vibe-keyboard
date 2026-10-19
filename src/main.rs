mod application;
mod logging;
mod presentation;

use application::{handle_filter_query, handle_frontmost, handle_key_event, render_screen};
use signal_hook::consts::{SIGINT, SIGTERM};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use vibe_keyboard::{domain, infrastructure};
use vibe_keyboard::domain::navigation::NavigationStack;
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::infrastructure::frontmost;
use vibe_keyboard::infrastructure::persistence::DeviceState;
use vibe_keyboard::infrastructure::profile_store;
use vibe_keyboard::infrastructure::usb::{
    clear_all, device_init, keep_alive, read_event, wait_for_device, Device,
};

const PROFILE_POLL: Duration = Duration::from_secs(1);
const FRONTMOST_POLL: Duration = Duration::from_millis(500);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Startup is the only place that may write the default profile.
    let (profile, profile_error) =
        match profile_store::ensure_default(&profile_store::profile_path()) {
            Ok(profile) => (profile, None),
            Err(e) => (Profile::default_from_legacy(), Some(e)),
        };
    let page_count = profile.pages.len();
    let app_state = Arc::new(Mutex::new(presentation::tui::AppState::new(page_count)));
    let _log_guard = logging::init(Arc::clone(&app_state));
    if let Some(e) = profile_error {
        error!("profile: {e}; using built-in layout");
    }
    log_problems(&profile);
    app_state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .profile = profile;
    // `--headless`: run the device loop without the ratatui TUI, so no terminal is needed.
    let headless = std::env::args().any(|arg| arg == "--headless");

    // One shutdown mechanism: SIGTERM and SIGINT set this flag.
    let shutdown = Arc::new(AtomicBool::new(false));
    for sig in [SIGTERM, SIGINT] {
        signal_hook::flag::register(sig, Arc::clone(&shutdown))?;
    }

    let mut last_profile_mtime = profile_store::modified_at(&profile_store::profile_path());
    let mut last_profile_check = Instant::now();

    let dev_state_val = DeviceState::load();
    let brightness = dev_state_val.brightness;
    let mut handle = match Device::open() {
        Ok(dev) => {
            device_init(&dev, brightness);
            dev
        }
        Err(e) => {
            info!("waiting for device: {e}");
            match wait_for_device(&shutdown, brightness) {
                Some(dev) => dev,
                None => return Ok(()),
            }
        }
    };
    info!("device ready");

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
    let mut nav = NavigationStack::new(initial_page, page_count);
    render_screen(&nav, &handle, &app_state, &dev_state);
    info!("listening — press keys (11=back, 12=out, 13=fwd)");

    let mut last_heartbeat = Instant::now();
    let frontmost_rx = frontmost::watch(FRONTMOST_POLL);
    let mut was_on_main_page = nav.main_page().is_some();

    loop {
        if shutdown.load(Ordering::Relaxed) {
            info!("stopping");
            clear_all(&handle);
            break;
        }

        if last_profile_check.elapsed() >= PROFILE_POLL {
            last_profile_check = Instant::now();
            let mtime = profile_store::modified_at(&profile_store::profile_path());
            if mtime != last_profile_mtime {
                last_profile_mtime = mtime;
                reload_profile(&app_state, &mut nav, &handle, &dev_state);
            }
        }

        while let Ok(bundle_id) = frontmost_rx.try_recv() {
            handle_frontmost(&bundle_id, &mut nav, &handle, &app_state, &dev_state);
        }

        let confirmed_query = {
            let mut s = app_state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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

        let read_lost = match read_event(&handle, Duration::from_millis(500)) {
            Ok(Some(data)) => {
                if data.len() >= 11 {
                    handle_key_event(&data, &handle, &mut nav, &app_state, &dev_state);
                }
                false
            }
            Ok(None) => false,
            Err(e) => {
                warn!("USB read error: {e}");
                true
            }
        };

        if last_heartbeat.elapsed() >= Duration::from_secs(8) {
            keep_alive(&handle);
            last_heartbeat = Instant::now();
        }

        // Back on a main page after a sub-screen: the app in front may have changed meanwhile.
        let on_main_page = nav.main_page().is_some();
        if on_main_page && !was_on_main_page {
            if let Some(bundle_id) = frontmost::frontmost_bundle_id() {
                handle_frontmost(&bundle_id, &mut nav, &handle, &app_state, &dev_state);
            }
        }
        was_on_main_page = on_main_page;

        if read_lost || handle.has_failed() {
            warn!("device disconnected");
            // Release the stale handle before waiting for the device to come back.
            drop(handle);
            let brightness = dev_state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .brightness;
            let Some(new_handle) = wait_for_device(&shutdown, brightness) else {
                break;
            };
            handle = new_handle;
            // Re-render the current screen from the same page images the editor previews.
            render_screen(&nav, &handle, &app_state, &dev_state);
            info!("device reconnected");
            last_heartbeat = Instant::now();
        }
    }

    Ok(())
}

/// Log each semantic problem in the profile. The layout still loads; bad keys do nothing.
fn log_problems(profile: &Profile) {
    for problem in profile.problems() {
        warn!("profile: {problem}");
    }
}

/// Load the edited profile into the running app. A broken or missing file keeps the
/// current layout. The controller never writes the file here.
fn reload_profile(
    app_state: &Arc<Mutex<presentation::tui::AppState>>,
    nav: &mut NavigationStack,
    handle: &Device,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    match profile_store::read(&profile_store::profile_path()) {
        Ok(Some(profile)) => {
            let count = profile.pages.len();
            log_problems(&profile);
            {
                let mut s = app_state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                s.total_pages = count;
                s.profile = profile;
                s.auto_page.clamp(count);
            }
            nav.set_main_pages(count);
            info!("profile reloaded: {count} page(s)");
            render_screen(nav, handle, app_state, dev_state);
        }
        Ok(None) => info!("profile.toml is missing; keeping current layout"),
        Err(e) => error!("profile reload failed, keeping current layout: {e}"),
    }
}
