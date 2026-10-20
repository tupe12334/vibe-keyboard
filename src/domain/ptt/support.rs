//! Fake poster and helpers for the guard tests.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use super::guard::PttGuard;
use super::{KeyEvent, KeyPoster, Probe};

pub(super) type Log = Rc<RefCell<Vec<(i32, KeyEvent)>>>;

pub(super) struct Fake(pub(super) Log);

impl KeyPoster for Fake {
    fn post(&mut self, pid: i32, event: KeyEvent) {
        self.0.borrow_mut().push((pid, event));
    }
}

pub(super) const SPACE: u16 = 49;
pub(super) const APP: i32 = 10;
pub(super) const OTHER: i32 = 20;

pub(super) fn ok(front: i32) -> Probe {
    Probe {
        front: Some(front),
        trusted: true,
    }
}

pub(super) fn setup() -> (PttGuard<Fake>, Log) {
    let log: Log = Rc::new(RefCell::new(Vec::new()));
    (PttGuard::new(Fake(Rc::clone(&log))), log)
}

pub(super) fn up_count(log: &Log) -> usize {
    log.borrow().iter().filter(|(_, e)| !e.down).count()
}

pub(super) fn start_hold(guard: &mut PttGuard<Fake>, key: u8, now: Instant) -> u64 {
    let generation = guard.begin(key);
    assert!(guard.start(generation, key, SPACE, 0, ok(APP), now));
    generation
}
