use std::path::PathBuf;

use super::ImageKind;

use eframe::egui;

use vibe_keyboard::infrastructure::profile_store;

impl super::Editor {
    /// Image source for one key: none, a file, an embedded icon, or an installed app's icon.
    pub(super) fn image_picker(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        image: &mut Option<PathBuf>,
    ) {
        let current = ImageKind::of(image.as_deref());
        let mut chosen = current;
        egui::ComboBox::from_label("Image")
            .selected_text(current.label())
            .show_ui(ui, |ui| {
                for kind in ImageKind::ALL {
                    ui.selectable_value(&mut chosen, kind, kind.label());
                }
            });
        if chosen != current {
            *image = match chosen {
                ImageKind::None => None,
                // Empty placeholders: the key shows its label until a value is chosen.
                ImageKind::File => Some(PathBuf::new()),
                ImageKind::Icon => Some(PathBuf::from("icon:back")),
                ImageKind::App => Some(PathBuf::from("app:")),
            };
        }
        match ImageKind::of(image.as_deref()) {
            ImageKind::None => {}
            ImageKind::File => {
                ui.horizontal(|ui| {
                    if ui.button("Browse...").clicked() {
                        let picked = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg"])
                            .pick_file();
                        if let Some(path) = picked {
                            match profile_store::import_image(&self.root, &path) {
                                Ok(relative) => *image = Some(relative),
                                Err(e) => self.status = e,
                            }
                        }
                    }
                    if let Some(path) = image.as_ref() {
                        ui.monospace(path.display().to_string());
                    }
                });
            }
            ImageKind::Icon => self.icon_grid(ui, ctx, image),
            ImageKind::App => self.app_picker(ui, image),
        }
    }
}
