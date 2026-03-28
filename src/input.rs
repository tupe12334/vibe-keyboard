use image::DynamicImage;
use rusb::{Context, DeviceHandle};
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::images::generate_terminal_image;
use crate::navigation::Navigator;
use crate::state::DeviceState;
use crate::tui;
use crate::usb::{clear_all, send_button_image};

pub fn raw_to_logical(raw: u8) -> Option<u8> {
    match raw {
        0x0D => Some(1),  0x0A => Some(2),  0x07 => Some(3),  0x04 => Some(4),  0x01 => Some(5),
        0x0E => Some(6),  0x0B => Some(7),  0x08 => Some(8),  0x05 => Some(9),  0x02 => Some(10),
        0x0F => Some(11), 0x0C => Some(12), 0x09 => Some(13), 0x06 => Some(14), 0x03 => Some(15),
        _ => None,
    }
}

fn open_terminal() {
    Command::new("open")
        .arg("-a")
        .arg("Terminal")
        .spawn()
        .unwrap_or_else(|e| { eprintln!("Failed to open Terminal: {e}"); std::process::exit(1) });
}

pub fn activate_page(
    page: usize,
    handle: &DeviceHandle<Context>,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    state.lock().unwrap().current_page = page;
    {
        let mut ds = dev_state.lock().unwrap();
        ds.current_page = page;
        ds.save();
    }
    clear_all(handle);
    match page {
        0 => {
            send_button_image(handle, 2, DynamicImage::ImageRgb8(generate_terminal_image()));
            state.lock().unwrap().push_log("page 0: terminal".into());
        }
        1 => {
            state.lock().unwrap().push_log("page 1: (empty)".into());
        }
        _ => {}
    }
}

pub fn handle_key_event(
    buf: &[u8],
    handle: &DeviceHandle<Context>,
    nav: &mut Navigator,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let raw_id = buf[9];
    let state_byte = buf[10];
    if raw_id == 0x00 || raw_id == 0xFF { return; }
    let state_str = match state_byte { 1 => "pressed", 2 => "released", _ => return };
    let key = match raw_to_logical(raw_id) {
        Some(k) => k,
        None => return,
    };
    {
        let mut s = state.lock().unwrap();
        s.pressed_key = if state_byte == 1 { Some(key) } else { None };
        if state_byte == 1 { s.push_log(format!("key {:2}  {state_str}", key)); }
    }
    if state_byte != 1 { return; }
    match key {
        11 => {
            let page = nav.back();
            state.lock().unwrap().push_log(format!("← back → page {}", page + 1));
            activate_page(page, handle, state, dev_state);
        }
        12 => {
            let page = nav.forward();
            state.lock().unwrap().push_log(format!("→ forward → page {}", page + 1));
            activate_page(page, handle, state, dev_state);
        }
        _ => {
            if nav.current() == 0 && key == 2 {
                state.lock().unwrap().push_log("opening Terminal".into());
                open_terminal();
            }
        }
    }
}
