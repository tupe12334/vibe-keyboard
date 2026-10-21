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

/// No hardware: writes drop, reads time out. Keeps tests off the real deck.
#[cfg(test)]
pub(crate) struct DetachedIo;

#[cfg(test)]
impl HidIo for DetachedIo {
    fn write(&self, packet: &[u8]) -> HidResult<usize> {
        Ok(packet.len())
    }

    fn read_timeout(&self, _buf: &mut [u8], _timeout_ms: i32) -> HidResult<usize> {
        Ok(0)
    }
}
