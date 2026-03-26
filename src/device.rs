use ajazz_sdk::{list_devices, new_hidapi, Ajazz, Kind};
use hidapi::HidApi;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::info;

/// Print all connected Ajazz devices to stdout.
pub fn list() -> Result<()> {
    let hid = new_hidapi().context("Failed to initialise HID API")?;
    let devices = list_devices(&hid);

    if devices.is_empty() {
        println!("No Ajazz devices found.");
    } else {
        println!("Found {} device(s):", devices.len());
        for (kind, serial) in &devices {
            println!("  {kind:?}  —  serial: {serial}");
        }
    }
    Ok(())
}

/// Immediately set brightness on the first (or named) device.
pub fn quick_brightness(value: u8, serial: Option<String>) -> Result<()> {
    let hid = new_hidapi().context("Failed to initialise HID API")?;
    let device = open(&hid, serial)?;
    device.set_brightness(value).context("set_brightness failed")?;
    info!("Brightness → {value}");
    Ok(())
}

/// Immediately put an image on a single button.
pub fn quick_image(key: u8, image_path: PathBuf, serial: Option<String>) -> Result<()> {
    let hid = new_hidapi().context("Failed to initialise HID API")?;
    let device = open(&hid, serial)?;
    let image = image::open(&image_path)
        .with_context(|| format!("Cannot open image {image_path:?}"))?;
    device.set_button_image(key, image).context("set_button_image failed")?;
    device.flush().context("flush failed")?;
    info!("Image set on button {key}");
    Ok(())
}

/// Immediately write a boot logo to the device.
pub fn quick_logo(image_path: PathBuf, serial: Option<String>) -> Result<()> {
    let hid = new_hidapi().context("Failed to initialise HID API")?;
    let device = open(&hid, serial)?;
    let image = image::open(&image_path)
        .with_context(|| format!("Cannot open image {image_path:?}"))?;
    device.set_logo_image(image).context("set_logo_image failed")?;
    device.flush().context("flush failed")?;
    info!("Boot logo written");
    Ok(())
}

// ── internal helpers ──────────────────────────────────────────────────────────

pub(crate) fn open_with(
    hid: &HidApi,
    serial: Option<String>,
) -> Result<(Ajazz, Kind, String)> {
    let devices = list_devices(hid);

    if devices.is_empty() {
        anyhow::bail!(
            "No Ajazz devices found — is the device connected? \
             On Linux, check your udev rules."
        );
    }

    let (kind, found_serial) = match &serial {
        Some(s) => devices
            .into_iter()
            .find(|(_, ser)| ser == s)
            .ok_or_else(|| anyhow::anyhow!("No device with serial '{s}' found"))?,
        None => devices.into_iter().next().unwrap(),
    };

    info!("Connecting to {kind:?} (serial: {found_serial})");
    let device = Ajazz::connect(hid, kind.clone(), &found_serial)
        .context("Failed to connect to device")?;

    Ok((device, kind, found_serial))
}

fn open(hid: &HidApi, serial: Option<String>) -> Result<Ajazz> {
    open_with(hid, serial).map(|(d, _, _)| d)
}
