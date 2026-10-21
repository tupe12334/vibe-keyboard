//! The HID calls `Device` makes. Tests put a fake behind `HidIo` instead of a real deck.
use hidapi::{HidDevice, HidResult};

/// Write and read one HID handle.
pub(crate) trait HidIo: Send {
    fn write(&self, packet: &[u8]) -> HidResult<usize>;
    fn read_timeout(&self, buf: &mut [u8], timeout_ms: i32) -> HidResult<usize>;
}

impl HidIo for HidDevice {
    fn write(&self, packet: &[u8]) -> HidResult<usize> {
        Self::write(self, packet)
    }

    fn read_timeout(&self, buf: &mut [u8], timeout_ms: i32) -> HidResult<usize> {
        Self::read_timeout(self, buf, timeout_ms)
    }
}
