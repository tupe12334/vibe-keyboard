use eframe::egui;

use vibe_keyboard::domain::profile::{format_color, parse_color, SideConfig};

use super::widgets::widget_editor;

impl super::Editor {
    /// Edit the side screen of the current page: label, colors, image. Add or remove it.
    pub(super) fn side_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Side screen");
        let page = self.page;
        let Some(original) = self
            .session
            .profile
            .pages
            .get(page)
            .and_then(|p| p.side.clone())
        else {
            ui.label("This page has no side screen.");
            if ui.button("Add side screen").clicked() {
                self.session
                    .profile
                    .set_side(page, Some(SideConfig::default()));
            }
            return;
        };
        let mut edited = original.clone();
        let ctx = ui.ctx().clone();
        ui.horizontal(|ui| {
            ui.label("Label");
            ui.text_edit_singleline(&mut edited.label);
        });
        ui.horizontal(|ui| {
            ui.label("Background");
            let mut rgb = parse_color(&edited.bg).unwrap_or([20, 20, 20]);
            if egui::color_picker::color_edit_button_srgb(ui, &mut rgb).changed() {
                edited.bg = format_color(rgb);
            }
            ui.label("Text");
            let mut rgb = parse_color(&edited.fg).unwrap_or([230, 230, 230]);
            if egui::color_picker::color_edit_button_srgb(ui, &mut rgb).changed() {
                edited.fg = format_color(rgb);
            }
        });
        self.image_picker(ui, &ctx, &mut edited.image);
        widget_editor(ui, &mut edited.widgets);
        ui.separator();
        let remove = ui.button("Remove side screen").clicked();
        // Write only real changes, so an untouched panel never dirties the profile.
        if remove {
            self.session.profile.set_side(page, None);
        } else if edited != original {
            self.session.profile.set_side(page, Some(edited));
        }
    }

    /// Margins for the band the bezel hides, on keys and on the side screen. Shaded in previews.
    pub(super) fn display_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Display");
        ui.label("Hidden bezel band, in px. Shaded in the previews.");
        let display = &mut self.session.profile.display;
        ui.add(egui::Slider::new(&mut display.key_inset.top, 0..=40).text("Key top"));
        ui.add(egui::Slider::new(&mut display.key_inset.bottom, 0..=40).text("Key bottom"));
        ui.add(egui::Slider::new(&mut display.side_inset.top, 0..=200).text("Side top"));
        ui.add(egui::Slider::new(&mut display.side_inset.bottom, 0..=200).text("Side bottom"));
    }
}
