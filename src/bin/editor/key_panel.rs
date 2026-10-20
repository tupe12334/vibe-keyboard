use eframe::egui;

use vibe_keyboard::domain::profile::{format_color, parse_color, Action, ActionKind};

use super::action_form::action_fields;
use super::RED;

impl super::Editor {
    pub(super) fn key_panel(&mut self, ui: &mut egui::Ui) {
        if self.side_open {
            self.side_panel(ui);
            return;
        }
        let Some(key) = self.key else {
            ui.label("Click a key to edit it.");
            return;
        };
        let page_count = self.session.profile.pages.len();
        let original = self
            .session
            .profile
            .key_at(self.page, key)
            .cloned()
            .unwrap_or_default();
        let mut edited = original.clone();
        let mut clear = false;

        ui.heading(format!("Key {key}"));
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
        let ctx = ui.ctx().clone();
        self.image_picker(ui, &ctx, &mut edited.image);

        let mut kind = edited.action.kind();
        egui::ComboBox::from_label("Action")
            .selected_text(kind.label())
            .show_ui(ui, |ui| {
                for option in ActionKind::ALL {
                    ui.selectable_value(&mut kind, option, option.label());
                }
            });
        if kind != edited.action.kind() {
            edited.action = Action::default_of(kind);
        }
        action_fields(ui, &mut edited.action, page_count);
        if let Some(problem) = edited.action.problem(page_count) {
            ui.colored_label(RED, problem);
        }

        // The hold action runs while the key is held past `[input] hold_ms`.
        let held_kind = edited.hold.as_ref().map_or(ActionKind::None, Action::kind);
        let mut hold_kind = held_kind;
        egui::ComboBox::from_label("Hold action")
            .selected_text(hold_kind.label())
            .show_ui(ui, |ui| {
                for option in ActionKind::ALL {
                    ui.selectable_value(&mut hold_kind, option, option.label());
                }
            });
        if hold_kind != held_kind {
            edited.hold = (hold_kind != ActionKind::None).then(|| Action::default_of(hold_kind));
        }
        if let Some(hold) = edited.hold.as_mut() {
            action_fields(ui, hold, page_count);
        }
        if let Some(problem) = edited
            .hold
            .as_ref()
            .and_then(|hold| hold.problem(page_count))
        {
            ui.colored_label(RED, problem);
        }

        ui.separator();
        if ui.button("Clear key").clicked() {
            clear = true;
        }

        // Write only real changes, so an untouched panel never dirties the profile.
        if clear {
            self.session.profile.clear_key(self.page, key);
        } else if edited != original {
            self.session.profile.set_key(self.page, key, edited);
        }
    }
}
