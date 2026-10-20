use image::RgbImage;

use super::{Plugin, PluginHost};
use crate::infrastructure::images::label_tile;

/// Opens the Moadim routines list.
pub struct Moadim;

impl Plugin for Moadim {
    fn name(&self) -> &'static str {
        "moadim"
    }
    fn label(&self) -> &'static str {
        "Moadim"
    }
    fn tile(&self) -> Option<RgbImage> {
        Some(label_tile("MOADIM", [28, 44, 84], [230, 230, 230]))
    }
    fn run(&self, host: &mut dyn PluginHost) {
        host.open_routines();
    }
}
