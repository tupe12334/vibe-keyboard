use crate::infrastructure::system_metrics::Disk;

pub(super) fn percent(value: u8) -> String {
    format!("{value}%")
}

pub(super) fn temperature_text(celsius: Option<f32>) -> String {
    celsius.map_or_else(|| "N/A".to_string(), |t| format!("{t:.0}C"))
}

pub(super) fn free_text(disk: Option<Disk>) -> String {
    disk.map_or_else(|| "N/A".to_string(), |d| free_space(d.free))
}

/// Decimal units, as Finder shows them: `1.2T` from a terabyte, else whole `G`.
fn free_space(bytes: u64) -> String {
    const GB: f64 = 1e9;
    const TB: f64 = 1e12;
    let value = bytes as f64;
    if value >= TB {
        format!("{:.1}T", value / TB)
    } else {
        format!("{:.0}G", value / GB)
    }
}
