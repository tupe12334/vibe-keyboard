use image::RgbImage;

use super::{Plugin, PluginHost};
use crate::domain::navigation::Screen;
use crate::infrastructure::images::generate_centy_image;

/// Opens the Centy project list. Does nothing when there are no projects.
pub struct CentyProjects;

impl Plugin for CentyProjects {
    fn name(&self) -> &'static str {
        "centy_projects"
    }
    fn label(&self) -> &'static str {
        "Centy projects"
    }
    fn tile(&self) -> Option<RgbImage> {
        Some(generate_centy_image())
    }
    fn run(&self, host: &mut dyn PluginHost) {
        host.set_loading(true);
        let projects = host.fetch_centy_projects();
        host.set_loading(false);
        if !projects.is_empty() {
            host.show(Screen::CentyProjectList {
                projects,
                page: 0,
                filter: None,
            });
        }
    }
}
