use eframe::egui::{self, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Vec2};

use vibe_keyboard::domain::keys::is_nav_key;
use vibe_keyboard::domain::profile::inner_rect;

use vibe_keyboard::infrastructure::usb::DECK_IMAGE_SIZE;

use super::previews::shade_hidden;
use super::{cell_size, GAP, GRID};

impl super::Editor {
    pub(super) fn grid(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> Option<u8> {
        let mut clicked = None;
        let cell = cell_size(ui.available_size());
        egui::Grid::new("keys").spacing([GAP, GAP]).show(ui, |ui| {
            for row in GRID {
                for key in row {
                    let response = self.key_cell(ui, ctx, key, cell);
                    if response.clicked() && !is_nav_key(key) {
                        clicked = Some(key);
                    }
                }
                ui.end_row();
            }
        });
        clicked
    }

    pub(super) fn key_cell(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        key: u8,
        cell: f32,
    ) -> egui::Response {
        let size = Vec2::splat(cell);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let texture = self.preview(ctx, key);
        let painter = ui.painter_at(rect);
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(texture.id(), rect, uv, Color32::WHITE);
        let inset = self.session.profile.display.key_inset;
        let visible = inner_rect(
            DECK_IMAGE_SIZE,
            DECK_IMAGE_SIZE,
            inset.top,
            inset.bottom,
            inset.left,
            inset.right,
        );
        shade_hidden(&painter, rect, (DECK_IMAGE_SIZE, DECK_IMAGE_SIZE), visible);
        // Key number in a small dark box, so it reads over any image.
        let number =
            egui::Rect::from_min_size(rect.left_top() + Vec2::new(2.0, 2.0), Vec2::new(18.0, 14.0));
        painter.rect_filled(number, 3.0, Color32::from_black_alpha(160));
        painter.text(
            number.center(),
            Align2::CENTER_CENTER,
            key.to_string(),
            FontId::monospace(10.0),
            Color32::WHITE,
        );
        if self.key == Some(key) {
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(2.0, Color32::LIGHT_BLUE),
                StrokeKind::Inside,
            );
        }
        response
    }
}
