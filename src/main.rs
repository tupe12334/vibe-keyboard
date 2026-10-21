use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, PoisonError};

use signal_hook::consts::{SIGINT, SIGTERM};
use tracing::{info, warn};
use vibe_keyboard::controller::{self, ControllerEvent, RunOptions};
use vibe_keyboard::domain::accessibility::{needs_accessibility, push_to_talk_warning};
use vibe_keyboard::infrastructure::controller_lock::{self, AcquireError};
use vibe_keyboard::infrastructure::keyboard::accessibility_trusted;
use vibe_keyboard::infrastructure::login_path::apply_login_path;
use vibe_keyboard::infrastructure::usb::calibrate::calibrate;
use vibe_keyboard::logging;
use vibe_keyboard::presentation::tui::AppState;

#[allow(
    clippy::print_stderr,
    reason = "the duplicate-start message is shown in the terminal before exit"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // First, before any thread or child process. A login start has a minimal PATH.
    let login_path_dirs = apply_login_path();
    // One controller per deck. Taken before logging (which truncates app.log) and before any
    // device access. The guard lives until exit, so the lock stays held for the whole run.
    let _controller_lock = match controller::claim() {
        Ok(lock) => lock,
        Err(AcquireError::Held) => {
            let lock_path = controller_lock::default_lock_path();
            eprintln!("{}", controller::held_message(&lock_path));
            // Login item (launchd KeepAlive SuccessfulExit=false) restarts only on non-zero
            // exit. A duplicate start is not a crash, so exit 0 and stop the restart loop.
            // Interactive runs exit 1 so the caller sees the failure.
            if std::env::args().any(|arg| arg == "--headless") {
                return Ok(());
            }
            std::process::exit(1);
        }
        Err(AcquireError::Io(e)) => return Err(format!("controller lock: {e}").into()),
    };
    // Hidden: draw each logical key number on its key, then exit. Checks key positions.
    if std::env::args().any(|arg| arg == "--calibrate") {
        return calibrate();
    }

    let state = Arc::new(Mutex::new(AppState::new(0)));
    let _log_guard = logging::init(Arc::clone(&state));
    info!("PATH extended at login: {login_path_dirs:?}");
    controller::load_profile(&state);
    // Push-to-talk does nothing without Accessibility. Say so once at startup.
    let uses = needs_accessibility(&state.lock().unwrap_or_else(PoisonError::into_inner).profile);
    if let Some(text) = push_to_talk_warning(accessibility_trusted(), uses) {
        warn!("{text}");
    }

    // One shutdown mechanism: SIGTERM and SIGINT set this flag.
    let shutdown = Arc::new(AtomicBool::new(false));
    for sig in [SIGTERM, SIGINT] {
        signal_hook::flag::register(sig, Arc::clone(&shutdown))?;
    }

    // `--headless`: run the device loop without the ratatui TUI, so no terminal is needed.
    let headless = std::env::args().any(|arg| arg == "--headless");
    controller::run(RunOptions {
        headless,
        shutdown,
        state,
        on_event: Box::new(|_: ControllerEvent| {}),
    })
}
