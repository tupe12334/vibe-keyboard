use eframe::egui::{self, Color32, Rect, Sense, Stroke, StrokeKind};

use vibe_keyboard::infrastructure::images::{safe_rect, side_image, SIDE_HEIGHT, SIDE_WIDTH};
use vibe_keyboard::infrastructure::system_metrics;

use super::previews::{shade_hidden, texture_from, SidePreview};
use super::{side_strip_size, GAP};

impl super::Editor {
    /// Texture of the side screen of the current page. Re-rendered only when its config changes.
    /// Re-rendered when its config or the latest system sample changes. Widget pages ask for
    /// a repaint every sample, so the values stay live.
    pub(super) fn side_texture(&mut self, ctx: &egui::Context) -> egui::TextureHandle {
        let config = self
            .session
            .profile
            .pages
            .get(self.page)
            .and_then(|p| p.side.clone());
        let inset = self.session.profile.display.side_inset;
        let metrics = system_metrics::snapshot(&self.metrics);
        if config.as_ref().is_some_and(|c| !c.widgets.is_empty()) {
            ctx.request_repaint_after(system_metrics::SAMPLE_INTERVAL);
        }
        if let Some(cached) = &self.side_preview {
            if cached.config == config && cached.metrics == metrics && cached.inset == inset {
                return cached.texture.clone();
            }
        }
        let img = side_image(config.as_ref(), &self.root, &metrics, &inset);
        let texture = texture_from(ctx, "side", &img);
        self.side_preview = Some(SidePreview {
            config,
            inset,
            metrics,
            texture: texture.clone(),
        });
        texture
    }

    /// Tall side preview beside the keys, with the hidden bezel band shaded. True when clicked.
    pub(super) fn side_strip(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let size = side_strip_size(ui.available_height());
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let texture = self.side_texture(ctx);
        let painter = ui.painter_at(rect);
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(texture.id(), rect, uv, Color32::WHITE);
        let inset = self.session.profile.display.side_inset;
        let visible = safe_rect(&inset);
        shade_hidden(&painter, rect, (SIDE_WIDTH, SIDE_HEIGHT), visible);
        if self.side_open {
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(2.0, Color32::LIGHT_BLUE),
                StrokeKind::Inside,
            );
        }
        ui.add_space(GAP);
        response.clicked()
    }
}
