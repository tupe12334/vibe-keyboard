use std::time::{Duration, SystemTime};

/// Time since the tty last printed. The device's modification time moves on every write to the
/// terminal. `None` when the device cannot be read.
pub(super) fn tty_age(tty: &str) -> Option<Duration> {
    let modified = std::fs::metadata(format!("/dev/{tty}"))
        .ok()?
        .modified()
        .ok()?;
    Some(
        SystemTime::now()
            .duration_since(modified)
            .unwrap_or_default(),
    )
}
