use std::path::PathBuf;

use eframe::egui::{self, Vec2};

use vibe_keyboard::infrastructure::images::{icon_image, ICON_NAMES};

use super::previews::texture_from;

impl super::Editor {
    /// Icon pack as clickable thumbnails. The selected icon is highlighted.
    pub(super) fn icon_grid(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        image: &mut Option<PathBuf>,
    ) {
        let selected = image
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            for name in ICON_NAMES.iter().copied() {
                let Some(texture) = self.icon_texture(ctx, name) else {
                    continue;
                };
                let thumb = egui::Image::new((texture.id(), Vec2::splat(40.0)));
                let button =
                    egui::Button::image(thumb).selected(selected == format!("icon:{name}"));
                if ui.add(button).on_hover_text(name).clicked() {
                    *image = Some(PathBuf::from(format!("icon:{name}")));
                }
            }
        });
    }

    /// Texture of one icon-pack image, cached per name.
    pub(super) fn icon_texture(
        &mut self,
        ctx: &egui::Context,
        name: &'static str,
    ) -> Option<egui::TextureHandle> {
        if let Some(texture) = self.icon_textures.get(name) {
            return Some(texture.clone());
        }
        let img = icon_image(name, 64)?;
        let texture = texture_from(ctx, name, &img);
        self.icon_textures.insert(name, texture.clone());
        Some(texture)
    }
}
