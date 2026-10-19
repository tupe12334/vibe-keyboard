pub mod app_icon;
// No caller in the binary yet; the app-aware PR will call `watch`. Remove this allow then.
#[allow(
    dead_code,
    reason = "no caller yet; the app-aware PR will call `watch`"
)]
pub mod frontmost;
pub mod images;
pub mod persistence;
pub mod usb;
