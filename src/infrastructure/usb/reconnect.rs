use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::warn;

use super::Device;

const RETRY_INTERVAL: Duration = Duration::from_secs(1);

/// Log failed attempt 1, then every 10th, so a long unplug stays quiet.
const fn should_log_failure(failures: u32) -> bool {
    failures % 10 == 1
}

/// Retry `Device::open` every second until it succeeds or `shutdown` is set.
pub fn wait_for_device(shutdown: &AtomicBool) -> Option<Device> {
    let mut failures = 0u32;
    while !shutdown.load(Ordering::Relaxed) {
        match Device::open() {
            Ok(dev) => return Some(dev),
            Err(e) => {
                failures += 1;
                if should_log_failure(failures) {
                    warn!("reconnect attempt {failures} failed: {e}");
                }
            }
        }
        std::thread::sleep(RETRY_INTERVAL);
    }
    None
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
