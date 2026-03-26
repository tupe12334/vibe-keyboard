use rusb::{Context, DeviceHandle, UsbContext};
use std::process::Command;
use std::time::Duration;

const VID: u16 = 0x0300;
const PID: u16 = 0x3010;
const INTERFACE: u8 = 0;
const INTERFACE2: u8 = 1;
const ENDPOINT_IN: u8 = 0x82;   // interface 0, vendor HID
const ENDPOINT_IN2: u8 = 0x81;  // interface 1, keyboard HID
const ENDPOINT_OUT: u8 = 0x03;
const PACKET_SIZE: usize = 512;
const TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_millis(500);

const CMD_PREFIX: [u8; 5] = [0x43, 0x52, 0x54, 0x00, 0x00];
const CMD_CONNECT_BODY: [u8; 7] = [0x43, 0x4F, 0x4E, 0x4E, 0x45, 0x43, 0x54];

fn build_command(body: &[u8]) -> Vec<u8> {
    let mut pkt = vec![0u8; 1024];
    pkt[..5].copy_from_slice(&CMD_PREFIX);
    pkt[5..5 + body.len()].copy_from_slice(body);
    pkt
}

fn open_terminal() {
    Command::new("open")
        .arg("-a")
        .arg("Terminal")
        .spawn()
        .unwrap_or_else(|e| { eprintln!("Failed to open Terminal: {e}"); std::process::exit(1) });
}

fn open_device(ctx: &Context) -> Result<DeviceHandle<Context>, rusb::Error> {
    ctx.open_device_with_vid_pid(VID, PID).ok_or_else(|| {
        eprintln!("Device {:04x}:{:04x} not found.", VID, PID);
        rusb::Error::NoDevice
    })
}

fn raw_to_logical(raw: u8) -> Option<u8> {
    match raw {
        0x01 => Some(11), 0x02 => Some(12), 0x03 => Some(13),
        0x04 => Some(14), 0x05 => Some(15), 0x06 => Some(6),
        0x07 => Some(7),  0x08 => Some(8),  0x09 => Some(9),
        0x0A => Some(10), 0x0B => Some(1),  0x0C => Some(2),
        0x0D => Some(3),  0x0E => Some(4),  0x0F => Some(5),
        _ => None,
    }
}

fn main() {
    let ctx = Context::new().expect("Failed to create libusb context");
    let handle = match open_device(&ctx) {
        Ok(h) => h,
        Err(_) => std::process::exit(1),
    };
    println!("[init] device opened");

    for iface in [INTERFACE, INTERFACE2] {
        match handle.detach_kernel_driver(iface) {
            Ok(()) => println!("[init] iface {iface}: dext detached"),
            Err(rusb::Error::NotFound) => println!("[init] iface {iface}: no driver"),
            Err(e) => eprintln!("[init] iface {iface} detach: {e}"),
        }
        match handle.claim_interface(iface) {
            Ok(()) => println!("[init] iface {iface}: claimed"),
            Err(e) => eprintln!("[init] iface {iface} claim: {e}"),
        }
    }

    // Query firmware version to confirm control pipe works.
    let mut fw_buf = [0u8; 64];
    match handle.read_control(0xA1, 0x01, 0x0100, 0x0000, &mut fw_buf, TIMEOUT) {
        Ok(n) => {
            let s = std::str::from_utf8(&fw_buf[..n]).unwrap_or("<non-utf8>");
            println!("[init] firmware: {s:?}");
        }
        Err(e) => eprintln!("[init] firmware query: {e}"),
    }

    // SET_INTERFACE resets endpoint data toggles and re-arms the pipes.
    for iface in [INTERFACE, INTERFACE2] {
        match handle.set_alternate_setting(iface, 0) {
            Ok(()) => println!("[init] set_alternate_setting iface {iface} ok"),
            Err(e) => eprintln!("[init] set_alternate_setting iface {iface}: {e}"),
        }
    }

    // Clear any stalled endpoints left over from dext eviction.
    for ep in [ENDPOINT_IN, ENDPOINT_IN2, ENDPOINT_OUT] {
        match handle.clear_halt(ep) {
            Ok(()) => println!("[init] clear_halt {ep:#04x} ok"),
            Err(e) => eprintln!("[init] clear_halt {ep:#04x}: {e}"),
        }
    }

    let connect_pkt = build_command(&CMD_CONNECT_BODY);
    match handle.write_interrupt(ENDPOINT_OUT, &connect_pkt, TIMEOUT) {
        Ok(n) => println!("[init] CONNECT ok ({n}b)"),
        Err(e) => eprintln!("[init] CONNECT: {e}"),
    }

    // Try GET_REPORT via control transfer — some devices route input this way.
    let mut report_buf = [0u8; PACKET_SIZE];
    match handle.read_control(0xA1, 0x01, 0x0100, INTERFACE as u16, &mut report_buf, TIMEOUT) {
        Ok(n) if n > 0 => {
            let end = report_buf[..n].iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
            println!("[init] GET_REPORT: {:02x?}", &report_buf[..end.max(1)]);
        }
        Ok(_) => println!("[init] GET_REPORT: empty"),
        Err(e) => eprintln!("[init] GET_REPORT: {e}"),
    }

    println!("[init] listening — press keys (30s window per poll)\n");

    let mut buf0 = [0u8; PACKET_SIZE];
    let mut buf1 = [0u8; 8];
    let mut heartbeat = 0u32;

    loop {
        // Poll vendor HID endpoint — try interrupt then bulk.
        let r0 = handle.read_interrupt(ENDPOINT_IN, &mut buf0, READ_TIMEOUT)
            .or_else(|_| handle.read_bulk(ENDPOINT_IN, &mut buf0, READ_TIMEOUT));
        match r0 {
            Ok(0) => {}
            Ok(n) => {
                let end = buf0[..n].iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
                if end > 0 {
                    println!("[ep82] {:02x?}", &buf0[..end]);
                    handle_key_event(&buf0);
                }
            }
            Err(rusb::Error::Timeout) => {}
            Err(e) => eprintln!("[ep82] error: {e}"),
        }

        // Also poll keyboard HID endpoint.
        let r1 = handle.read_interrupt(ENDPOINT_IN2, &mut buf1, READ_TIMEOUT)
            .or_else(|_| handle.read_bulk(ENDPOINT_IN2, &mut buf1, READ_TIMEOUT));
        match r1 {
            Ok(0) => {}
            Ok(n) => println!("[ep81] {:02x?}", &buf1[..n]),
            Err(rusb::Error::Timeout) => {}
            Err(e) => eprintln!("[ep81] error: {e}"),
        }

        heartbeat += 1;
        if heartbeat % 150 == 0 {
            // ~15 s heartbeat
            handle.write_interrupt(ENDPOINT_OUT, &connect_pkt, TIMEOUT).ok();
        }
    }
}

fn handle_key_event(buf: &[u8]) {
    let raw_id = buf[9];
    let state  = buf[10];
    if raw_id == 0x00 { return; }
    if raw_id == 0xFF { println!("[ack]"); return; }
    let state_str = match state { 1 => "pressed", 2 => "released", s => { println!("state={s:#04x}"); return; } };
    match raw_to_logical(raw_id) {
        Some(key) => {
            println!("key {key:2}  {state_str}");
            if key == 1 && state == 1 {
                println!("→ opening Terminal");
                open_terminal();
            }
        }
        None => println!("unknown raw_id={raw_id:#04x} state={state:#04x}"),
    }
}
