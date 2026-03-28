mod commands;
mod image;

pub use commands::{clear_all, device_init, keep_alive, read_event, set_brightness};
pub use image::send_button_image;

use std::time::Duration;

pub const VID: u16 = 0x0300;
pub const PID: u16 = 0x3010;
pub const EP_OUT: u8 = 0x03;
pub const EP_IN: u8 = 0x82;
pub const PACKET: usize = 512;
pub const TIMEOUT: Duration = Duration::from_millis(1000);
