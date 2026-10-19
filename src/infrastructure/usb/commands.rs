use rusb::{Context, DeviceHandle};
use std::time::Duration;
use tracing::{error, warn};

use super::{CMD_PACKET, EP_IN, EP_OUT, PACKET, TIMEOUT};

fn make_cmd(body: &[u8]) -> [u8; CMD_PACKET] {
    let mut pkt = [0u8; CMD_PACKET];
    pkt[..5].copy_from_slice(b"CRT\0\0");
    pkt[5..5 + body.len()].copy_from_slice(body);
    pkt
}

pub fn write_cmd(handle: &DeviceHandle<Context>, body: &[u8]) {
    if let Err(e) = handle.write_interrupt(EP_OUT, &make_cmd(body), TIMEOUT) {
        error!("write_cmd failed: {e}");
    }
}

/// Reset endpoint data toggles and clear any stalls left over from dext eviction.
/// Must be called after `claim_interface` and before the first write.
pub fn reset_endpoints(handle: &DeviceHandle<Context>) {
    let _ = handle.set_alternate_setting(0, 0);
    let _ = handle.clear_halt(EP_IN);
    let _ = handle.clear_halt(EP_OUT);
}

/// Init order the official `StreamDock` app sends (`device.md`, captured 2026-10-20).
pub fn device_init(handle: &DeviceHandle<Context>, brightness: u8) {
    write_cmd(handle, b"DIS");
    set_brightness(handle, brightness);
    write_cmd(handle, b"QUCMD\x11\x11\x00\x11\x00\x11");
    clear_all(handle);
    refresh(handle);
    write_cmd(handle, b"CONNECT");
}

pub fn set_brightness(handle: &DeviceHandle<Context>, percent: u8) {
    write_cmd(handle, &lig_body(percent));
}

pub fn clear_all(handle: &DeviceHandle<Context>) {
    write_cmd(handle, &CLE_BODY);
}

/// Commit pending screen changes (`STP`).
pub fn refresh(handle: &DeviceHandle<Context>) {
    write_cmd(handle, b"STP");
}

const fn lig_body(percent: u8) -> [u8; 6] {
    [b'L', b'I', b'G', 0, 0, percent]
}

const CLE_BODY: [u8; 7] = [b'C', b'L', b'E', 0, 0, 0, 0xff];

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected full packet: `CRT\0\0` + body, zero-padded to `CMD_PACKET`.
    fn expected(body: &[u8]) -> Vec<u8> {
        let mut v = b"CRT\0\0".to_vec();
        v.extend_from_slice(body);
        v.resize(CMD_PACKET, 0);
        v
    }

    #[test]
    fn qucmd_bytes() {
        let body = b"QUCMD\x11\x11\x00\x11\x00\x11";
        assert_eq!(make_cmd(body).as_slice(), expected(body).as_slice());
    }

    #[test]
    fn cle_bytes() {
        assert_eq!(
            make_cmd(&CLE_BODY).as_slice(),
            expected(b"CLE\0\0\0\xff").as_slice()
        );
    }

    #[test]
    fn stp_bytes() {
        assert_eq!(make_cmd(b"STP").as_slice(), expected(b"STP").as_slice());
    }

    #[test]
    fn lig_brightness_15_at_offset_10() {
        let pkt = make_cmd(&lig_body(15));
        assert_eq!(pkt.as_slice(), expected(b"LIG\0\0\x0f").as_slice());
        assert_eq!(pkt[10], 0x0f);
    }
}
