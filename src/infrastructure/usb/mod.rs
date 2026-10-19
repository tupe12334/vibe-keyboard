mod commands;
mod image;
mod reconnect;

pub use commands::{clear_all, device_init, keep_alive, read_event};
pub use image::{send_button_image, send_side_image};
pub use reconnect::wait_for_device;

use hidapi::{HidApi, HidDevice, HidResult};
use std::cell::Cell;

pub const VID: u16 = 0x0300;
pub const PID: u16 = 0x3010;
/// Vendor interface (no report IDs). Skips the boot keyboard interface.
const USAGE_PAGE: u16 = 0xFFA0;
/// Primary usage on the vendor page. Usage `0x02` is a second collection on the same page.
const USAGE: u16 = 0x01;
/// Output payload size of the vendor interface (`MaxOutputReportSize`).
pub const PACKET: usize = 1024;
/// Output report: leading `0x00` (no report ID) + one `PACKET`.
pub const CMD_PACKET: usize = PACKET + 1;

/// Open HID handle to the Stream Dock vendor interface.
pub struct Device {
    hid: HidDevice,
    /// Set on the first failed write or read. Writers swallow errors, so the loop checks this.
    failed: Cell<bool>,
}

impl Device {
    /// Open the vendor interface in non-exclusive mode, so no root is needed.
    pub fn open() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| format!("hidapi init: {e}"))?;
        #[cfg(target_os = "macos")]
        api.set_open_exclusive(false);
        let info = api
            .device_list()
            .find(|d| {
                d.vendor_id() == VID
                    && d.product_id() == PID
                    && d.usage_page() == USAGE_PAGE
                    && d.usage() == USAGE
            })
            .ok_or("Device not found — is it plugged in?")?;
        let hid = info
            .open_device(&api)
            .map_err(|e| format!("Failed to open HID device: {e}"))?;
        Ok(Self {
            hid,
            failed: Cell::new(false),
        })
    }

    /// Write one output report. `packet[0]` must be `0x00` (no report ID).
    pub fn write(&self, packet: &[u8]) -> HidResult<usize> {
        let res = self.hid.write(packet);
        if res.is_err() {
            self.failed.set(true);
        }
        res
    }

    /// Read one input report. Returns `Ok(0)` on timeout.
    pub fn read_timeout(&self, buf: &mut [u8], timeout_ms: i32) -> HidResult<usize> {
        let res = self.hid.read_timeout(buf, timeout_ms);
        if res.is_err() {
            self.failed.set(true);
        }
        res
    }

    /// True once any write or read on this handle has failed. Stays true for this handle.
    pub const fn has_failed(&self) -> bool {
        self.failed.get()
    }
}
