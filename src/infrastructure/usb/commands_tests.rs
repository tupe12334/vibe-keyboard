use super::*;

/// Expected output report: `0x00` + `CRT\0\0` + body, zero-padded to `CMD_PACKET`.
fn expected(body: &[u8]) -> Vec<u8> {
    let mut v = vec![0x00];
    v.extend_from_slice(b"CRT\0\0");
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
    let pkt = make_cmd(b"STP");
    assert_eq!(pkt.len(), 1025);
    assert_eq!(pkt.as_slice(), expected(b"STP").as_slice());
}

#[test]
fn lig_brightness_15_at_offset_10() {
    let pkt = make_cmd(&lig_body(15));
    assert_eq!(pkt.as_slice(), expected(b"LIG\0\0\x0f").as_slice());
    // Device frame offset 10 = packet index 11 (report ID byte is not sent).
    assert_eq!(pkt[11], 0x0f);
}
