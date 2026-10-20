use std::path::PathBuf;

use eframe::egui;

impl super::Editor {
    /// Installed-app icon by bundle id: pick a running app, or type the id.
    pub(super) fn app_picker(&mut self, ui: &mut egui::Ui, image: &mut Option<PathBuf>) {
        let bundle = image
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let bundle = bundle.strip_prefix("app:").unwrap_or_default().to_string();
        ui.monospace(if bundle.is_empty() {
            "(no app chosen)".to_string()
        } else {
            bundle.clone()
        });
        let mut pick = None;
        egui::ComboBox::from_id_salt("image-app")
            .selected_text("Pick a running app")
            .show_ui(ui, |ui| {
                for (name, id) in &self.running {
                    if ui
                        .selectable_label(false, format!("{name} ({id})"))
                        .clicked()
                    {
                        pick = Some(id.clone());
                    }
                }
            });
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.image_app_text);
            if ui.button("Use").clicked() {
                pick = Some(self.image_app_text.trim().to_string());
            }
        });
        if let Some(id) = pick.filter(|id| !id.is_empty()) {
            *image = Some(PathBuf::from(format!("app:{id}")));
        }
    }
}
