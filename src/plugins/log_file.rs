use image::RgbImage;
use tracing::info;

use super::{Plugin, PluginHost};
use crate::infrastructure::images::generate_log_file_image;

/// Opens the app log file (`app.log`) in VS Code.
pub struct LogFile;

impl Plugin for LogFile {
    fn name(&self) -> &'static str {
        "log_file"
    }
    fn label(&self) -> &'static str {
        "Log file"
    }
    fn tile(&self) -> Option<RgbImage> {
        Some(generate_log_file_image())
    }
    fn run(&self, host: &mut dyn PluginHost) {
        info!("opening log file in VS Code");
        host.open_log_file();
    }
}
