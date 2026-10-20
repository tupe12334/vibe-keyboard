use image::RgbImage;
use tracing::info;

use super::{Plugin, PluginHost};
use crate::infrastructure::images::generate_claude_image;

/// Opens Claude Code in a Terminal window.
pub struct Claude;

impl Plugin for Claude {
    fn name(&self) -> &'static str {
        "claude"
    }
    fn label(&self) -> &'static str {
        "Claude"
    }
    fn tile(&self) -> Option<RgbImage> {
        Some(generate_claude_image())
    }
    fn run(&self, host: &mut dyn PluginHost) {
        info!("opening Claude in Terminal");
        host.open_claude_terminal();
    }
}
