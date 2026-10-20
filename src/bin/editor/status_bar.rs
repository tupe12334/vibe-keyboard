use eframe::egui;
use std::time::Instant;

use vibe_keyboard::domain::profile_rules;

use super::RED;

impl super::Editor {
    pub(super) fn status_bar(&mut self, ui: &mut egui::Ui, now: Instant) {
        if let Some(error) = &self.load_error {
            ui.colored_label(RED, format!("profile.toml: {error}"));
        }
        if self.session.undo_available(now) {
            ui.horizontal(|ui| {
                ui.label("Changes discarded —");
                if ui.button("Undo").clicked() {
                    self.undo();
                }
            });
        } else {
            ui.label(self.status.as_str());
        }
        let problems = profile_rules::save_problems(&self.session.profile);
        if !problems.is_empty() {
            ui.collapsing(
                format!("{} problem(s) in the layout", problems.len()),
                |ui| {
                    for problem in &problems {
                        ui.colored_label(RED, problem);
                    }
                },
            );
        }
    }
}
