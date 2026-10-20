//! A Moadim routine as the device shows it, and the status that picks its tile colour.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// One routine from the Moadim daemon. Only the fields the device reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Routine {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    pub running: bool,
    /// Consecutive failed runs. Zero means the last run did not fail.
    pub failures: u32,
    /// Unix seconds of the last scheduled or manual trigger.
    pub last_trigger: Option<u64>,
}

/// Tile colour. Checked in this order: running, failed, enabled, disabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoutineStatus {
    Running,
    Failed,
    Enabled,
    Disabled,
}

impl Routine {
    pub const fn status(&self) -> RoutineStatus {
        if self.running {
            RoutineStatus::Running
        } else if self.failures > 0 {
            RoutineStatus::Failed
        } else if self.enabled {
            RoutineStatus::Enabled
        } else {
            RoutineStatus::Disabled
        }
    }
}

impl RoutineStatus {
    /// Word for the button description.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Failed => "Last run failed",
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }
}

/// What the routines page knows about the daemon.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MoadimList {
    /// No answer yet. The first refresh is still running.
    Pending,
    /// The daemon did not answer in time.
    Offline,
    Online(Vec<Routine>),
}

/// A short label that replaces one key's tile until `until`.
#[derive(Clone, Debug)]
pub struct Flash {
    pub key: u8,
    pub label: &'static str,
    pub until: Instant,
}

/// How long after a TRIGGER press a routine ignores further TRIGGER presses.
pub const TRIGGER_DEBOUNCE: Duration = Duration::from_secs(2);

/// Remembers when each routine last got a TRIGGER press, so a double tap runs it once.
#[derive(Default)]
pub struct TriggerDebounce {
    last: HashMap<String, Instant>,
}

impl TriggerDebounce {
    /// True when a press at `now` should run routine `id`. Records the press when it should.
    pub fn allow(&mut self, id: &str, now: Instant) -> bool {
        if self
            .last
            .get(id)
            .is_some_and(|&t| now.duration_since(t) < TRIGGER_DEBOUNCE)
        {
            return false;
        }
        self.last.insert(id.to_owned(), now);
        true
    }
}

#[cfg(test)]
#[path = "moadim_tests.rs"]
mod tests;
