mod commands;
mod image;

pub use commands::{
    clear_all, device_init, keep_alive, read_event, reset_endpoints, set_brightness,
};
pub use image::send_button_image;

use std::time::Duration;

pub const VID: u16 = 0x0300;
pub const PID: u16 = 0x3010;
pub const EP_OUT: u8 = 0x03;
pub const EP_IN: u8 = 0x82;
pub const PACKET: usize = 512;
pub const CMD_PACKET: usize = 517; // 5-byte CRT prefix + 512-byte body
pub const TIMEOUT: Duration = Duration::from_secs(1);
