use eframe::egui;

impl super::Editor {
    /// Bundle ids that show the page at `index` when they are in front.
    pub(super) fn app_rules(&mut self, ui: &mut egui::Ui, index: usize) {
        ui.label("Show this page when these apps are in front:");
        let mut remove = None;
        for (i, app) in self.session.profile.pages[index].apps.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.monospace(app);
                if ui.small_button("remove").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            self.session.profile.pages[index].apps.remove(i);
        }
        let mut add = None;
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_app);
            if ui.button("Add").clicked() {
                add = Some(self.new_app.trim().to_string());
            }
        });
        egui::ComboBox::from_id_salt("running-apps")
            .selected_text("Pick a running app")
            .show_ui(ui, |ui| {
                for (name, id) in &self.running {
                    if ui
                        .selectable_label(false, format!("{name} ({id})"))
                        .clicked()
                    {
                        add = Some(id.clone());
                    }
                }
            });
        if ui.button("Refresh running apps").clicked() {
            self.start_running_refresh();
        }
        if let Some(id) = add {
            let apps = &mut self.session.profile.pages[index].apps;
            if !id.is_empty() && !apps.contains(&id) {
                apps.push(id);
            }
            self.new_app.clear();
        }
    }
}
