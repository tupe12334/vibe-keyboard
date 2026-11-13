//! Reads of the frontmost app's pid and bundle id. Two `lsappinfo` reads share one time limit.
//! The process spawns live in `frontmost/os.rs`.

use std::time::Duration;

use super::{parse_bundle_id, parse_pid};

/// One field of the frontmost app, from two reads: `front` for its ASN, then `info -only <field>`.
/// Both reads share one `limit`. The second gets only the time the first one left.
fn front_field(
    limit: Duration,
    elapsed: &dyn Fn() -> Duration,
    run: &dyn Fn(&[&str], Duration) -> Option<String>,
    field: &str,
) -> Option<String> {
    let asn = run(&["front"], limit.saturating_sub(elapsed()))?;
    run(
        &["info", "-only", field, &asn],
        limit.saturating_sub(elapsed()),
    )
}

/// Pid of the frontmost app, read within `limit` as `front_field` does.
pub(super) fn pid_within(
    limit: Duration,
    elapsed: &dyn Fn() -> Duration,
    run: &dyn Fn(&[&str], Duration) -> Option<String>,
) -> Option<i32> {
    parse_pid(&front_field(limit, elapsed, run, "pid")?)
}

/// Bundle id of the frontmost app, read within `limit` as `front_field` does.
pub(super) fn bundle_within(
    limit: Duration,
    elapsed: &dyn Fn() -> Duration,
    run: &dyn Fn(&[&str], Duration) -> Option<String>,
) -> Option<String> {
    parse_bundle_id(&front_field(limit, elapsed, run, "bundleid")?)
}

#[cfg(test)]
#[path = "front_fields_tests.rs"]
mod tests;
