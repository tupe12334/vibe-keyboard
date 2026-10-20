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
