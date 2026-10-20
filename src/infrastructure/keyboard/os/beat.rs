//! The two loops of a hold. The repeat beat only takes the lock. The focus probe spawns
//! `lsappinfo`, which can take 100+ ms under load, so it runs on its own thread.

use std::thread;
use std::time::{Duration, Instant};

use tracing::warn;

use super::hold::{lock, read_probe};
use crate::domain::ptt::{deadline, next_index};

/// How often a hold re-checks the frontmost app and Accessibility.
const PROBE_EVERY: Duration = Duration::from_millis(250);
/// The iTerm2 tty read is ignored this long after a hold starts. Focus has only just landed, and a
/// read taken then can lag. The move rule still needs two misses in a row.
const SESSION_GRACE: Duration = Duration::from_millis(500);
/// `QOS_CLASS_USER_INTERACTIVE`, from `<sys/qos.h>`.
const QOS_CLASS_USER_INTERACTIVE: u32 = 0x21;

extern "C" {
    fn pthread_set_qos_class_self_np(qos_class: u32, relative_priority: i32) -> i32;
}

/// Post a repeat key down every 30 ms until the hold ends. The beats sit on an absolute
/// grid from the first one, so a slow beat never shifts the later ones.
pub(super) fn beat(generation: u64) {
    // SAFETY: takes a QoS class and a priority by value, touches no memory, and only changes this thread.
    let status = unsafe { pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0) };
    if status != 0 {
        warn!("push to talk: repeat thread stays at default QoS (status {status})");
    }
    let start = Instant::now();
    let mut next = 1;
    loop {
        thread::sleep(deadline(start, next).saturating_duration_since(Instant::now()));
        let now = Instant::now();
        if !lock().pump(generation, now) {
            break;
        }
        next = next_index(start, next, now);
    }
}

/// Probe focus and Accessibility every `PROBE_EVERY` until the hold ends.
pub(super) fn watch_focus(generation: u64, tty: &Option<String>) {
    let started = Instant::now();
    loop {
        thread::sleep(PROBE_EVERY);
        let mut probe = read_probe(tty.as_deref());
        if started.elapsed() < SESSION_GRACE {
            probe.session_moved = false;
        }
        if !lock().observe(generation, probe) {
            break;
        }
    }
}
