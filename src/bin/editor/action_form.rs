use eframe::egui;

use vibe_keyboard::domain::profile::{Action, BUILTIN_NAMES};

/// Inputs for the fields of one action type.
pub(super) fn action_fields(ui: &mut egui::Ui, action: &mut Action, page_count: usize) {
    match action {
        Action::None => {}
        Action::OpenUrl { url } => {
            ui.horizontal(|ui| {
                ui.label("URL");
                ui.text_edit_singleline(url);
            });
        }
        Action::RunCommand { command } => {
            ui.horizontal(|ui| {
                ui.label("Command");
                ui.text_edit_singleline(command);
            });
        }
        Action::OpenApp { name } => {
            ui.horizontal(|ui| {
                ui.label("App name");
                ui.text_edit_singleline(name);
            });
        }
        Action::Hotkey { keys } => {
            ui.horizontal(|ui| {
                ui.label("Keys");
                ui.text_edit_singleline(keys);
            });
            ui.weak("e.g. cmd+shift+4");
        }
        Action::GoToPage { page } => {
            ui.horizontal(|ui| {
                ui.label("Page (0-based)");
                ui.add(egui::DragValue::new(page).range(0..=page_count.saturating_sub(1)));
            });
        }
        Action::Builtin { name } => {
            egui::ComboBox::from_label("Built-in")
                .selected_text(name.as_str())
                .show_ui(ui, |ui| {
                    for option in BUILTIN_NAMES {
                        ui.selectable_value(name, option.to_string(), option);
                    }
                });
        }
        Action::PushToTalk { key } => {
            let mut text = key.clone().unwrap_or_default();
            ui.horizontal(|ui| {
                ui.label("Key");
                ui.text_edit_singleline(&mut text);
            });
            ui.weak("Empty means Space, the Claude Code dictation key.");
            *key = (!text.trim().is_empty()).then_some(text);
        }
    }
}
