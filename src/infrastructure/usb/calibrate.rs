//! `--calibrate`: draws keys 1-15 with their numbers and the visible rect outlined.

use tracing::warn;

use super::{device_init, send_button_image, Device};
use crate::domain::profile::KeyInset;
use crate::infrastructure::images::calibration_image;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::profile_store;

/// Draw logical keys 1–15 (their numbers) on the deck, then return.
pub fn calibrate() -> Result<(), Box<dyn std::error::Error>> {
    let dev = Device::open()?;
    let brightness = DeviceState::load().brightness;
    device_init(&dev, brightness);
    // The loaded profile's key inset, so the tiles show the visible rect the user tunes.
    let inset = match profile_store::read(&profile_store::profile_path()) {
        Ok(profile) => profile.map(|p| p.display.key_inset).unwrap_or_default(),
        Err(e) => {
            warn!("profile: {e}; calibrating with default insets");
            KeyInset::default()
        }
    };
    for key in 1..=15u8 {
        let tile = calibration_image(key, &inset);
        send_button_image(&dev, key, &image::DynamicImage::ImageRgb8(tile));
    }
    Ok(())
}
