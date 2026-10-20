use eframe::egui;

use super::Preview;

use vibe_keyboard::infrastructure::images::{app_icon_generation, key_image};

impl super::Editor {
    /// Texture of the deck image for `key`. Re-rendered only when that key's config changes.
    pub(super) fn preview(&mut self, ctx: &egui::Context, key: u8) -> egui::TextureHandle {
        let config = self.session.profile.key_at(self.page, key).cloned();
        let inset = self.session.profile.display.key_inset;
        let generation = app_icon_generation();
        if let Some(cached) = self.previews.get(&key) {
            if cached.config == config && cached.generation == generation && cached.inset == inset {
                return cached.texture.clone();
            }
        }
        let img = key_image(key, config.as_ref(), &self.root, &inset);
        let size = [img.width() as usize, img.height() as usize];
        let rgba: Vec<u8> = img
            .pixels()
            .flat_map(|p| [p.0[0], p.0[1], p.0[2], 255])
            .collect();
        let color = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
        let texture = ctx.load_texture(format!("key-{key}"), color, egui::TextureOptions::LINEAR);
        self.previews.insert(
            key,
            Preview {
                config,
                inset,
                generation,
                texture: texture.clone(),
            },
        );
        texture
    }
}
