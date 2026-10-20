use eframe::egui::{self, Align2, Color32, Vec2};

use vibe_keyboard::infrastructure::profile_store;

impl super::Editor {
    /// Shown when the file changed on disk since it was loaded.
    pub(super) fn conflict_dialog(&mut self, ctx: &egui::Context) {
        let (mut reload, mut overwrite, mut keep) = (false, false, false);
        egui::Window::new("profile.toml changed on disk")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Another program changed the file after it was loaded.");
                ui.label("Reload discards your edits. Overwrite replaces the file.");
                ui.horizontal(|ui| {
                    reload |= ui.button("Reload").clicked();
                    overwrite |= ui.button("Overwrite").clicked();
                    keep |= ui.button("Keep editing").clicked();
                });
            });
        if reload {
            self.adopt_disk();
            self.status = "Reloaded profile.toml from disk.".into();
        } else if overwrite {
            self.write_file(true);
        } else if keep {
            self.disk_conflict = false;
        }
    }

    pub(super) fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Stream deck layout");
            if self.dirty() {
                ui.colored_label(Color32::YELLOW, "Unsaved changes");
            }
            let dirty = self.dirty();
            let can_save = dirty && self.load_error.is_none();
            if ui
                .add_enabled(can_save, egui::Button::new("Save"))
                .clicked()
            {
                self.save();
            }
            if ui
                .add_enabled(dirty, egui::Button::new("Discard changes"))
                .clicked()
            {
                self.discard();
            }
            ui.weak(profile_store::profile_path().display().to_string());
        });
    }

    /// Modal shown when the window closes with unsaved edits. Cancel is the default.
    pub(super) fn close_dialog(&mut self, ctx: &egui::Context) {
        let (escape, enter) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Enter),
            )
        });
        let (mut save, mut discard, mut cancel) = (false, false, escape || enter);
        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Save changes to profile.toml before closing?");
                ui.horizontal(|ui| {
                    save |= ui
                        .add_enabled(self.load_error.is_none(), egui::Button::new("Save"))
                        .clicked();
                    discard |= ui.button("Discard").clicked();
                    cancel |= ui.button("Cancel").clicked();
                });
            });
        if cancel {
            self.close_prompt = false;
        } else if discard || (save && self.save()) {
            self.close_prompt = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
