use image::RgbImage;
use tracing::info;

use super::{Plugin, PluginHost};
use crate::domain::navigation::Screen;
use crate::infrastructure::images::generate_numpad_entry_image;

/// Opens the number input page.
pub struct Numpad;

impl Plugin for Numpad {
    fn name(&self) -> &'static str {
        "numpad"
    }
    fn label(&self) -> &'static str {
        "Numpad"
    }
    fn tile(&self) -> Option<RgbImage> {
        Some(generate_numpad_entry_image())
    }
    fn run(&self, host: &mut dyn PluginHost) {
        info!("opening input number page");
        host.show(Screen::InputNumber {
            value: String::new(),
        });
    }
}
