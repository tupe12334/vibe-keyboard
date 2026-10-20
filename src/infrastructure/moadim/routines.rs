//! Parsing the daemon's routine list, and the labels the routines page shows.

use serde_json::Value;

use super::tool_text;
use crate::domain::moadim::Routine;

/// Routines from a `list_routines` response. `None` when the body is not a routine list.
pub fn parse_routines(body: &str) -> Option<Vec<Routine>> {
    let items: Vec<Value> = serde_json::from_str(&tool_text(body)?).ok()?;
    Some(items.iter().filter_map(routine_from).collect())
}

fn routine_from(item: &Value) -> Option<Routine> {
    Some(Routine {
        id: item.get("id")?.as_str()?.to_owned(),
        title: item.get("title")?.as_str()?.to_owned(),
        enabled: item.get("enabled")?.as_bool()?,
        running: item.get("is_running").and_then(Value::as_bool) == Some(true),
        failures: item
            .get("consecutive_failures")
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0),
        last_trigger: number(item, "last_manual_trigger_at")
            .max(number(item, "last_scheduled_trigger_at")),
    })
}

fn number(item: &Value, key: &str) -> Option<u64> {
    item.get(key)?.as_u64()
}

/// Short age of a unix time, such as `5M AGO`. `NO RUNS` when there is no time.
pub fn ago(then: Option<u64>, now: u64) -> String {
    let Some(then) = then else {
        return "NO RUNS".into();
    };
    let secs = now.saturating_sub(then);
    match secs {
        0..=3599 => format!("{}M AGO", secs / 60),
        3600..=86_399 => format!("{}H AGO", secs / 3600),
        _ => format!("{}D AGO", secs / 86_400),
    }
}

/// Text for the LAST RUN tile: the failure count after a failed run, else the age of the last trigger.
pub fn last_run_label(routine: &Routine, now: u64) -> String {
    if routine.failures > 0 {
        format!("FAIL x{}", routine.failures)
    } else {
        ago(routine.last_trigger, now)
    }
}
