//! Fake HID handle for the reconnect test: scripted reads, and counted reads and writes.
use crate::infrastructure::usb::{Device, HidIo};
use hidapi::{HidError, HidResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Reads and writes one fake handle made.
#[derive(Default)]
pub struct Counts {
    pub reads: usize,
    pub writes: usize,
}

/// A handle. Its first read fails when `unplug` is set. Otherwise the read times out and
/// sets `stop`, so the loop ends on its next pass.
struct FakeIo {
    counts: Arc<Mutex<Counts>>,
    unplug: bool,
    stop: Arc<AtomicBool>,
}

impl FakeIo {
    fn tally(&self) -> MutexGuard<'_, Counts> {
        self.counts.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl HidIo for FakeIo {
    fn write(&self, packet: &[u8]) -> HidResult<usize> {
        self.tally().writes += 1;
        Ok(packet.len())
    }

    fn read_timeout(&self, _buf: &mut [u8], _timeout_ms: i32) -> HidResult<usize> {
        self.tally().reads += 1;
        if self.unplug {
            return Err(HidError::HidApiError {
                message: "unplugged".to_string(),
            });
        }
        self.stop.store(true, Ordering::Relaxed);
        Ok(0)
    }
}

pub fn handle(unplug: bool, counts: &Arc<Mutex<Counts>>, stop: &Arc<AtomicBool>) -> Device {
    Device::from_io(Box::new(FakeIo {
        counts: Arc::clone(counts),
        unplug,
        stop: Arc::clone(stop),
    }))
}
