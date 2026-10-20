use image::RgbImage;
use tracing::info;

use super::{Plugin, PluginHost};
use crate::infrastructure::images::generate_vscode_config_image;

/// Opens the device state file (`state.toml`) in VS Code.
pub struct VscodeConfig;

impl Plugin for VscodeConfig {
    fn name(&self) -> &'static str {
        "vscode_config"
    }
    fn label(&self) -> &'static str {
        "VS Code config"
    }
    fn tile(&self) -> Option<RgbImage> {
        Some(generate_vscode_config_image())
    }
    fn run(&self, host: &mut dyn PluginHost) {
        info!("opening config in VS Code");
        host.open_config_in_vscode();
    }
}
