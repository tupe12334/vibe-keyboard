use image::RgbImage;
use tracing::info;

use super::{Plugin, PluginHost};
use crate::infrastructure::images::generate_terminal_image;

/// Opens a Terminal window.
pub struct Terminal;

impl Plugin for Terminal {
    fn name(&self) -> &'static str {
        "terminal"
    }
    fn label(&self) -> &'static str {
        "Terminal"
    }
    fn tile(&self) -> Option<RgbImage> {
        Some(generate_terminal_image())
    }
    fn run(&self, host: &mut dyn PluginHost) {
        info!("opening Terminal");
        host.open_terminal();
    }
}
