/// Daemon — the long-running process that:
///   1. Connects to the first available Ajazz device
///   2. Applies the initial config (brightness + button images + boot logo)
///   3. Reads button events and fires the configured actions
///   4. Watches the config file and hot-reloads when it changes
///
/// Threading model
/// ───────────────
/// HID I/O is synchronous and `DeviceStateReader` is !Send, so it stays on a
/// dedicated std::thread.  Tokio handles config-file watching and action
/// execution.  Two channels bridge the two worlds:
///
///   cmd_tx  (std::sync::mpsc)  tokio → HID thread   DeviceCommand
///   evt_tx  (tokio::mpsc)      HID thread → tokio   DeviceEvent
use crate::actions;
use crate::config::{ButtonConfig, Config};
use ajazz_sdk::{list_devices, new_hidapi, Ajazz, AjazzInput};
use anyhow::{Context, Result};
use notify::Watcher;
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::time::Duration;
use tokio::sync::mpsc as tok_mpsc;
use tracing::{error, info, warn};

// ── channel message types ────────────────────────────────────────────────────

enum DeviceCommand {
    ApplyConfig(Config),
    Shutdown,
}

enum DeviceEvent {
    ButtonPressed(u8),
    ButtonReleased(u8),
    DeviceError(String),
}

// ── public entry point ───────────────────────────────────────────────────────

pub async fn run(config_path: PathBuf) -> Result<()> {
    let config = Config::load(&config_path)?;
    info!("Config loaded from {config_path:?}");

    // tokio → HID thread
    let (cmd_tx, cmd_rx) = std_mpsc::channel::<DeviceCommand>();
    // HID thread → tokio
    let (evt_tx, mut evt_rx) = tok_mpsc::unbounded_channel::<DeviceEvent>();

    let initial = config.clone();
    let hid_cmd_tx = cmd_tx.clone();

    // Spawn the dedicated HID thread
    let hid_thread = std::thread::Builder::new()
        .name("hid-io".into())
        .spawn(move || {
            if let Err(e) = hid_loop(cmd_rx, evt_tx, initial) {
                error!("HID thread exited with error: {e:#}");
            }
        })
        .context("Failed to spawn HID thread")?;

    // Watch the config file for changes
    let (notify_tx, mut notify_rx) = tok_mpsc::channel::<()>(8);
    let _watcher = start_file_watcher(&config_path, notify_tx)?;

    let mut current_config = config;

    info!("Daemon running — press Ctrl+C to stop");

    loop {
        tokio::select! {
            // ── button / device events ────────────────────────────────────
            Some(evt) = evt_rx.recv() => match evt {
                DeviceEvent::ButtonPressed(key) => {
                    info!("Button {key} pressed");
                    if let Some(btn) = find_button(&current_config, key) {
                        if let Some(action) = btn.action.clone() {
                            tokio::task::spawn_blocking(move || {
                                if let Err(e) = actions::execute(&action) {
                                    error!("Action on button {key} failed: {e:#}");
                                }
                            });
                        }
                    }
                }
                DeviceEvent::ButtonReleased(key) => {
                    info!("Button {key} released");
                }
                DeviceEvent::DeviceError(msg) => {
                    error!("Device error: {msg}");
                    break;
                }
            },

            // ── config file changed ───────────────────────────────────────
            Some(()) = notify_rx.recv() => {
                // Debounce: drain any follow-up events within 200 ms
                let deadline = tokio::time::Instant::now() + Duration::from_millis(200);
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep_until(deadline) => break,
                        Some(()) = notify_rx.recv() => {} // swallow
                    }
                }

                info!("Config file changed — reloading");
                match Config::load(&config_path) {
                    Ok(new_cfg) => {
                        current_config = new_cfg.clone();
                        if hid_cmd_tx.send(DeviceCommand::ApplyConfig(new_cfg)).is_err() {
                            error!("HID thread has gone away");
                            break;
                        }
                    }
                    Err(e) => warn!("Failed to reload config: {e:#}"),
                }
            }

            else => break,
        }
    }

    let _ = cmd_tx.send(DeviceCommand::Shutdown);
    drop(cmd_tx);
    hid_thread.join().ok();
    info!("Daemon stopped");
    Ok(())
}

// ── HID thread ───────────────────────────────────────────────────────────────

fn hid_loop(
    cmd_rx: std_mpsc::Receiver<DeviceCommand>,
    evt_tx: tok_mpsc::UnboundedSender<DeviceEvent>,
    initial_config: Config,
) -> Result<()> {
    let hid = new_hidapi().context("Failed to initialise HID API")?;
    let devices = list_devices(&hid);

    if devices.is_empty() {
        anyhow::bail!(
            "No Ajazz devices found — is the device connected? \
             On Linux, check your udev rules."
        );
    }

    let (kind, serial) = devices.into_iter().next().unwrap();
    info!("Connecting to {kind:?} (serial: {serial})");

    let mut device = Ajazz::connect(&hid, kind, &serial)
        .context("Failed to connect to device")?;

    apply_config(&mut device, &initial_config);

    // Track button states manually so we can detect press vs. release from
    // the raw AjazzInput::ButtonStateChange(Vec<bool>) snapshot.
    let mut prev_states: Vec<bool> = Vec::new();

    loop {
        // ── drain any pending commands (non-blocking) ─────────────────────
        loop {
            match cmd_rx.try_recv() {
                Ok(DeviceCommand::ApplyConfig(cfg)) => apply_config(&mut device, &cfg),
                Ok(DeviceCommand::Shutdown) => {
                    info!("HID thread shutting down");
                    return Ok(());
                }
                Err(std_mpsc::TryRecvError::Empty) => break,
                Err(std_mpsc::TryRecvError::Disconnected) => return Ok(()),
            }
        }

        // ── read one HID report ───────────────────────────────────────────
        // `read_input()` uses hidapi's read_timeout internally.  It returns
        // AjazzInput::NoData if the timeout expires with no report.
        match device.read_input(Some(Duration::from_millis(100))) {
            Ok(AjazzInput::ButtonStateChange(states)) => {
                for (i, &pressed) in states.iter().enumerate() {
                    let was = prev_states.get(i).copied().unwrap_or(false);
                    let key = i as u8;
                    if pressed && !was {
                        let _ = evt_tx.send(DeviceEvent::ButtonPressed(key));
                    } else if !pressed && was {
                        let _ = evt_tx.send(DeviceEvent::ButtonReleased(key));
                    }
                }
                prev_states = states;
            }
            Ok(AjazzInput::NoData) => {
                // Nothing received within the SDK's internal timeout window —
                // just loop back and check for commands again.
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(_) => {
                // Encoder events etc. — ignored for now
            }
            Err(e) => {
                let _ = evt_tx.send(DeviceEvent::DeviceError(e.to_string()));
                return Err(anyhow::anyhow!("HID read error: {e}"));
            }
        }
    }
}

// ── config application ────────────────────────────────────────────────────────

fn apply_config(device: &mut Ajazz, config: &Config) {
    info!(
        "Applying config — brightness: {}, buttons: {}",
        config.brightness,
        config.buttons.len()
    );

    // Brightness takes effect immediately (no flush needed)
    if let Err(e) = device.set_brightness(config.brightness) {
        warn!("set_brightness failed: {e}");
    }

    // Clear everything, then paint only the buttons that have images
    if let Err(e) = device.clear_all_button_images() {
        warn!("clear_all_button_images failed: {e}");
    }

    for btn in &config.buttons {
        if let Some(ref path) = btn.image {
            match image::open(path) {
                Ok(img) => {
                    if let Err(e) = device.set_button_image(btn.index, img) {
                        warn!("set_button_image({}) failed: {e}", btn.index);
                    }
                }
                Err(e) => warn!("Cannot open image {path:?}: {e}"),
            }
        }
    }

    if let Err(e) = device.flush() {
        warn!("flush failed: {e}");
    }

    // Boot logo (only when specified)
    if let Some(ref path) = config.boot_logo {
        match image::open(path) {
            Ok(img) => {
                if let Err(e) = device.set_logo_image(img) {
                    warn!("set_logo_image failed: {e}");
                } else if let Err(e) = device.flush() {
                    warn!("flush (logo) failed: {e}");
                } else {
                    info!("Boot logo written");
                }
            }
            Err(e) => warn!("Cannot open boot logo {path:?}: {e}"),
        }
    }

    info!("Config applied");
}

// ── file watcher ──────────────────────────────────────────────────────────────

fn start_file_watcher(
    config_path: &PathBuf,
    tx: tok_mpsc::Sender<()>,
) -> Result<impl notify::Watcher> {
    // Watch the parent directory so we catch atomic renames that editors use
    // when writing files.  Filter to the config filename only.
    let config_name = config_path
        .file_name()
        .map(|n| n.to_owned())
        .unwrap_or_default();
    let watch_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                let relevant = event
                    .paths
                    .iter()
                    .any(|p| p.file_name().map_or(false, |n| n == config_name));
                if relevant && (event.kind.is_modify() || event.kind.is_create()) {
                    let _ = tx.blocking_send(());
                }
            }
            Err(e) => warn!("File watcher error: {e}"),
        })
        .context("Failed to create file watcher")?;

    watcher
        .watch(&watch_dir, notify::RecursiveMode::NonRecursive)
        .context("Failed to watch config directory")?;

    Ok(watcher)
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn find_button<'a>(config: &'a Config, key: u8) -> Option<&'a ButtonConfig> {
    config.buttons.iter().find(|b| b.index == key)
}
