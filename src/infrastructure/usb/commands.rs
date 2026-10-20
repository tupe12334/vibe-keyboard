use hidapi::HidResult;
use std::time::Duration;
use tracing::error;

use super::{Device, CMD_PACKET};

/// Input report size. No report ID.
const IN_PACKET: usize = 512;

/// Output report: `0x00` (no report ID), then `CRT\0\0` + body, zero-padded.
fn make_cmd(body: &[u8]) -> [u8; CMD_PACKET] {
    let mut pkt = [0u8; CMD_PACKET];
    pkt[1..6].copy_from_slice(b"CRT\0\0");
    pkt[6..6 + body.len()].copy_from_slice(body);
    pkt
}

pub fn write_cmd(handle: &Device, body: &[u8]) {
    if let Err(e) = handle.write(&make_cmd(body)) {
        error!("write_cmd failed: {e}");
    }
}

/// Init order the official `StreamDock` app sends (`device.md`, captured 2026-10-20).
pub fn device_init(handle: &Device, brightness: u8) {
    write_cmd(handle, b"DIS");
    set_brightness(handle, brightness);
    write_cmd(handle, b"QUCMD\x11\x11\x00\x11\x00\x11");
    clear_all(handle);
    refresh(handle);
    write_cmd(handle, b"CONNECT");
}

pub fn set_brightness(handle: &Device, percent: u8) {
    write_cmd(handle, &lig_body(percent));
}

pub fn clear_all(handle: &Device) {
    write_cmd(handle, &CLE_BODY);
}

/// Commit pending screen changes (`STP`).
pub fn refresh(handle: &Device) {
    write_cmd(handle, b"STP");
}

const fn lig_body(percent: u8) -> [u8; 6] {
    [b'L', b'I', b'G', 0, 0, percent]
}

const CLE_BODY: [u8; 7] = [b'C', b'L', b'E', 0, 0, 0, 0xff];

pub fn keep_alive(handle: &Device) {
    write_cmd(handle, b"CONNECT");
}

/// `Ok(None)` on timeout, `Err` when the device read fails (e.g. unplugged).
pub fn read_event(handle: &Device, timeout: Duration) -> HidResult<Option<Vec<u8>>> {
    let mut buf = vec![0u8; IN_PACKET];
    let timeout_ms = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX);
    match handle.read_timeout(&mut buf, timeout_ms)? {
        0 => Ok(None),
        n => {
            buf.truncate(n);
            Ok(Some(buf))
        }
    }
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
