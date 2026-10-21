use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::warn;

use super::{device_init, DeckPort, Device};

pub(super) const RETRY_INTERVAL: Duration = Duration::from_secs(1);

/// Log failed attempt 1, then every 10th, so a long unplug stays quiet.
const fn should_log_failure(failures: u32) -> bool {
    failures % 10 == 1
}

/// Open and init as one attempt. Fails if the handle hits any I/O error during init.
fn open_and_init(port: &dyn DeckPort, brightness: u8) -> Result<Device, String> {
    let dev = port.open()?;
    device_init(&dev, brightness);
    if dev.has_failed() {
        return Err("device init I/O failed".to_string());
    }
    Ok(dev)
}

/// Retry open + init every second until one attempt succeeds or `shutdown` is set.
/// Sleeps before every attempt, so a handle that opens but fails at once cannot spin the loop.
pub fn wait_for_device(
    port: &dyn DeckPort,
    shutdown: &AtomicBool,
    brightness: u8,
) -> Option<Device> {
    let mut failures = 0u32;
    loop {
        std::thread::sleep(port.retry_interval());
        if shutdown.load(Ordering::Relaxed) {
            return None;
        }
        match open_and_init(port, brightness) {
            Ok(dev) => return Some(dev),
            Err(e) => {
                failures += 1;
                if should_log_failure(failures) {
                    warn!("reconnect attempt {failures} failed: {e}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::should_log_failure;

    #[test]
    fn logs_first_and_every_tenth_failure() {
        let logged: Vec<u32> = (1..=25).filter(|&n| should_log_failure(n)).collect();
        assert_eq!(logged, vec![1, 11, 21]);
    }
}
