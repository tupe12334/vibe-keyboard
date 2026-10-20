//! System metrics for the side screen: CPU use, free disk space and CPU temperature.
//!
//! This file holds the pure logic. Every sysinfo call lives in `system_metrics/os.rs`,
//! which runs a background sampler. The device loop only reads the snapshot.

// The `#[path]` keeps this working when the example includes this file directly.
#[path = "system_metrics/os.rs"]
mod os;

mod disks;
mod temperature;

use disks::{choose_disk, is_candidate_mount};
use temperature::hottest_cpu;
#[cfg(test)]
use temperature::is_cpu_sensor;

use std::sync::{Arc, Mutex};
use std::time::Duration;

pub use os::start;

/// How often the sampler refreshes the snapshot.
pub const SAMPLE_INTERVAL: Duration = Duration::from_secs(2);

/// Shared snapshot. `None` until the first sample lands.
pub type Shared = Arc<Mutex<Option<Metrics>>>;

/// Free and total bytes of one volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Disk {
    pub free: u64,
    pub total: u64,
}

/// One sample. `Default` is the state before the first sample: 0 % CPU, no disk, no sensor.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Metrics {
    /// Global CPU use, 0 to 100.
    pub cpu_percent: u8,
    /// Free space on the user volume. `None` if no volume matched.
    pub disk: Option<Disk>,
    /// Hottest CPU sensor, in degrees Celsius. `None` when the machine has no such sensor.
    pub temperature: Option<f32>,
}

/// Copy of the latest sample, or the default before the first one.
pub fn snapshot(shared: &Shared) -> Metrics {
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .unwrap_or_default()
}

/// Rounds a CPU reading to a whole percent and keeps it in 0..=100. NaN becomes 0.
pub const fn clamp_percent(value: f32) -> u8 {
    if value.is_nan() {
        return 0;
    }
    value.round().clamp(0.0, 100.0) as u8
}

/// Builds one sample from the raw values the sampler read.
pub fn build(cpu: f32, disks: &[(String, Disk)], readings: &[(String, f32)]) -> Metrics {
    let candidates: Vec<Disk> = disks
        .iter()
        .filter(|(mount, _)| is_candidate_mount(mount))
        .map(|(_, disk)| *disk)
        .collect();
    Metrics {
        cpu_percent: clamp_percent(cpu),
        disk: choose_disk(&candidates),
        temperature: hottest_cpu(readings),
    }
}

#[cfg(test)]
#[path = "system_metrics_tests.rs"]
mod tests;
