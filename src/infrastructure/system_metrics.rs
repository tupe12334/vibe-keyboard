//! System metrics for the side screen: CPU use, free disk space and CPU temperature.
//!
//! This file holds the pure logic. Every sysinfo call lives in `system_metrics/os.rs`,
//! which runs a background sampler. The device loop only reads the snapshot.

// The `#[path]` keeps this working when the example includes this file directly.
#[path = "system_metrics/os.rs"]
mod os;

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

/// Mount points that hold the user's data. On APFS, `/` is the sealed system snapshot and
/// `/System/Volumes/Data` is the writable volume. Both report the same container.
pub fn is_candidate_mount(mount: &str) -> bool {
    matches!(mount, "/" | "/System/Volumes/Data")
}

/// Picks the candidate with the most free space: the volume a user can still fill.
pub fn choose_disk(candidates: &[Disk]) -> Option<Disk> {
    candidates.iter().copied().max_by_key(|disk| disk.free)
}

/// Whether a sensor label reads a CPU die. Apple Silicon exposes `PMU tdie<N>` and
/// `PMU2 tdie<N>` (checked on an M-series Mac); AMD on Linux uses `Tdie`.
/// `tdev`, `tcal`, NAND and battery sensors are not the CPU.
pub fn is_cpu_sensor(label: &str) -> bool {
    let lower = label.to_ascii_lowercase();
    lower.contains("tdie") || lower.contains("cpu")
}

/// Plausible die temperature range. Outside it the sensor is reporting garbage.
const SANE_TEMP: std::ops::RangeInclusive<f32> = -40.0..=125.0;

/// Hottest plausible reading among CPU sensors. `None` when no CPU sensor reports one.
pub fn hottest_cpu(readings: &[(String, f32)]) -> Option<f32> {
    readings
        .iter()
        .filter(|(label, temp)| is_cpu_sensor(label) && SANE_TEMP.contains(temp))
        .map(|(_, temp)| *temp)
        .reduce(f32::max)
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
mod tests {
    use super::*;

    fn disk(free: u64) -> Disk {
        Disk { free, total: 1000 }
    }

    #[test]
    fn percent_is_rounded_and_clamped() {
        assert_eq!(clamp_percent(-3.0), 0);
        assert_eq!(clamp_percent(f32::NAN), 0);
        assert_eq!(clamp_percent(42.4), 42);
        assert_eq!(clamp_percent(42.6), 43);
        assert_eq!(clamp_percent(120.0), 100);
    }

    #[test]
    fn only_root_and_data_volumes_are_candidates() {
        assert!(is_candidate_mount("/"));
        assert!(is_candidate_mount("/System/Volumes/Data"));
        assert!(!is_candidate_mount("/Volumes/Backup"));
        assert!(!is_candidate_mount("/System/Volumes/Preboot"));
    }

    #[test]
    fn disk_choice_takes_the_most_free_space() {
        assert_eq!(choose_disk(&[]), None);
        assert_eq!(choose_disk(&[disk(10), disk(90), disk(40)]), Some(disk(90)));
    }

    #[test]
    fn hottest_cpu_reads_the_maximum_or_none() {
        assert_eq!(hottest_cpu(&[]), None);
        let readings = [
            ("PMU tdie1".to_string(), 71.4),
            ("PMU2 tdie3".to_string(), 51.5),
            ("PMU tdev6".to_string(), 99.0),
            ("NAND CH0 temp".to_string(), 80.0),
            ("PMU tdev4".to_string(), -9201.1),
        ];
        assert_eq!(hottest_cpu(&readings), Some(71.4));
        assert!(is_cpu_sensor("Tdie"));
        assert!(!is_cpu_sensor("PMU tcal"));
    }

    #[test]
    fn build_filters_volumes_and_keeps_missing_values_empty() {
        let disks = [
            ("/Volumes/Backup".to_string(), disk(999)),
            ("/".to_string(), disk(10)),
            ("/System/Volumes/Data".to_string(), disk(30)),
        ];
        let metrics = build(150.0, &disks, &[]);
        assert_eq!(metrics.cpu_percent, 100);
        assert_eq!(metrics.disk, Some(disk(30)));
        assert_eq!(metrics.temperature, None);
        assert_eq!(build(0.0, &[], &[]), Metrics::default());
    }

    #[test]
    fn snapshot_is_default_before_the_first_sample() {
        let shared: Shared = Arc::new(Mutex::new(None));
        assert_eq!(snapshot(&shared), Metrics::default());
        *shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Metrics {
            cpu_percent: 7,
            ..Metrics::default()
        });
        assert_eq!(snapshot(&shared).cpu_percent, 7);
    }
}
