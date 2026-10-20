//! The only sysinfo calls. Everything here is untested by the coverage gate; the
//! pure logic it feeds lives in `system_metrics.rs`.

use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::thread;

use sysinfo::{Components, Disks, System, MINIMUM_CPU_UPDATE_INTERVAL};

use super::{build, is_candidate_mount, Disk, Metrics, Shared, SAMPLE_INTERVAL};

/// Starts the background sampler and returns the snapshot it fills.
pub fn start() -> Shared {
    let shared: Shared = Arc::new(Mutex::new(None));
    let writer = Arc::clone(&shared);
    thread::spawn(move || {
        let mut probe = Probe::new();
        // CPU usage needs two samples at least `MINIMUM_CPU_UPDATE_INTERVAL` apart.
        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
        loop {
            if let Ok(metrics) = panic::catch_unwind(AssertUnwindSafe(|| probe.read())) {
                *writer
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(metrics);
            } else {
                tracing::error!("system metrics sample panicked");
            }
            thread::sleep(SAMPLE_INTERVAL);
        }
    });
    shared
}

/// Holds the sysinfo state between reads, so CPU usage is a delta over the interval.
struct Probe {
    sys: System,
    components: Components,
    disks: Disks,
}

impl Probe {
    fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        Self {
            sys,
            components: Components::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
        }
    }

    fn read(&mut self) -> Metrics {
        self.sys.refresh_cpu_usage();
        self.components.refresh(true);
        self.disks.refresh(true);
        let disks: Vec<(String, Disk)> = self
            .disks
            .list()
            .iter()
            .filter(|d| is_candidate_mount(&d.mount_point().to_string_lossy()))
            .map(|d| {
                let disk = Disk {
                    free: d.available_space(),
                    total: d.total_space(),
                };
                (d.mount_point().to_string_lossy().into_owned(), disk)
            })
            .collect();
        let readings: Vec<(String, f32)> = self
            .components
            .list()
            .iter()
            .filter_map(|c| c.temperature().map(|t| (c.label().to_string(), t)))
            .collect();
        build(self.sys.global_cpu_usage(), &disks, &readings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prints one live snapshot. Run: `cargo test system_metrics -- --ignored --nocapture`.
    #[test]
    #[allow(clippy::print_stdout, reason = "manual hardware probe")]
    #[ignore = "reads real hardware sensors"]
    fn print_one_snapshot() {
        let mut probe = Probe::new();
        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
        let metrics = probe.read();
        let labels: Vec<String> = probe
            .components
            .list()
            .iter()
            .map(|c| format!("{} = {:?}", c.label(), c.temperature()))
            .collect();
        println!("METRICS {metrics:?}");
        println!("SENSORS\n{}", labels.join("\n"));
    }
}
