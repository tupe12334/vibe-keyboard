use rusb::{Context, DeviceHandle};
use std::time::Duration;
use tracing::warn;

use super::{CMD_PACKET, EP_IN, EP_OUT, PACKET, TIMEOUT};

fn make_cmd(body: &[u8]) -> [u8; CMD_PACKET] {
    let mut pkt = [0u8; CMD_PACKET];
    pkt[..5].copy_from_slice(b"CRT\0\0");
    pkt[5..5 + body.len()].copy_from_slice(body);
    pkt
}

pub fn write_cmd(handle: &DeviceHandle<Context>, body: &[u8]) {
    handle
        .write_interrupt(EP_OUT, &make_cmd(body), TIMEOUT)
        .expect("write_cmd failed");
}

/// Reset endpoint data toggles and clear any stalls left over from dext eviction.
/// Must be called after claim_interface and before the first write.
pub fn reset_endpoints(handle: &DeviceHandle<Context>) {
    handle.set_alternate_setting(0, 0).ok();
    handle.clear_halt(EP_IN).ok();
    handle.clear_halt(EP_OUT).ok();
}

pub fn device_init(handle: &DeviceHandle<Context>) {
    write_cmd(handle, b"CONNECT");
    write_cmd(handle, b"DIS");
    write_cmd(handle, &[b'L', b'I', b'G', 0, 0, 0, 0]);
}

pub fn set_brightness(handle: &DeviceHandle<Context>, percent: u8) {
    write_cmd(handle, &[b'L', b'I', b'G', 0, 0, percent]);
}

pub fn clear_all(handle: &DeviceHandle<Context>) {
    write_cmd(handle, &[b'C', b'L', b'E', 0, 0, 0, 0xff]);
}

pub fn keep_alive(handle: &DeviceHandle<Context>) {
    write_cmd(handle, b"CONNECT");
}

pub fn read_event(handle: &DeviceHandle<Context>, timeout: Duration) -> Option<Vec<u8>> {
    let mut buf = vec![0u8; PACKET];
    match handle.read_interrupt(EP_IN, &mut buf, timeout) {
        Ok(_) => Some(buf),
        Err(rusb::Error::Timeout) => None,
        Err(e) => {
            warn!("USB read error: {e}");
            None
        }
    }
}
