// No caller in the binary yet; the app-aware PR will call `watch`. Remove this allow then.
#[allow(dead_code)]
pub mod frontmost;
pub mod app_icon;
pub mod images;
pub mod persistence;
pub mod usb;
