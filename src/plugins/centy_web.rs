use image::RgbImage;
use tracing::info;

use super::{Plugin, PluginHost};
use crate::infrastructure::images::generate_web_image;

/// Opens the Centy web app in Chrome.
pub struct CentyWeb;

impl Plugin for CentyWeb {
    fn name(&self) -> &'static str {
        "centy_web"
    }
    fn label(&self) -> &'static str {
        "Centy web"
    }
    fn tile(&self) -> Option<RgbImage> {
        Some(generate_web_image())
    }
    fn run(&self, host: &mut dyn PluginHost) {
        info!("opening Centy in Chrome");
        host.open_in_chrome("https://app.centy.io");
    }
}
