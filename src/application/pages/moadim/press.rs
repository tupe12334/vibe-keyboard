//! Routine presses: run the action on a worker thread and report its result on the next tick.

use std::time::Instant;

use super::{Done, MoadimRefresh};
use crate::infrastructure::moadim::{call_action, press_label, RoutineAction};

impl MoadimRefresh {
    /// Run routine action `verb` on routine `id` for a press on `key`. The worker thread reports
    /// the result on the next `tick`: `label` on success, `FAILED` otherwise. A TRIGGER press
    /// within the debounce window of the last one for the same routine does nothing.
    pub fn press(&mut self, key: u8, id: String, verb: RoutineAction, label: &'static str) {
        if verb == RoutineAction::Trigger && !self.debounce.allow(&id, Instant::now()) {
            return;
        }
        let tx = self.done_tx.clone();
        std::thread::spawn(move || {
            let label = press_label(call_action(verb, &id), label);
            let _ = tx.send(Done { key, label });
        });
    }
}
