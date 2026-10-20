//! The two loops of a hold. The repeat beat only takes the lock. The focus probe spawns
//! `lsappinfo`, which can take 100+ ms under load, so it runs on its own thread.

use std::thread;
use std::time::{Duration, Instant};

use super::hold::{lock, read_probe};

/// Gap between repeated key downs while held. Terminal key repeat is about 30 ms.
const REPEAT_EVERY: Duration = Duration::from_millis(30);
/// How often a hold re-checks the frontmost app and Accessibility.
const PROBE_EVERY: Duration = Duration::from_millis(250);

/// Post a repeat key down every `REPEAT_EVERY` until the hold ends.
pub(super) fn beat(generation: u64) {
    let mut next = Instant::now() + REPEAT_EVERY;
    loop {
        thread::sleep(next.saturating_duration_since(Instant::now()));
        next += REPEAT_EVERY;
        if !lock().pump(generation, Instant::now()) {
            break;
        }
    }
}

/// Probe focus and Accessibility every `PROBE_EVERY` until the hold ends.
pub(super) fn watch_focus(generation: u64, tty: &Option<String>) {
    loop {
        thread::sleep(PROBE_EVERY);
        let probe = read_probe(tty.as_deref());
        if !lock().observe(generation, probe) {
            break;
        }
    }
}
