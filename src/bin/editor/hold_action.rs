use eframe::egui;

use vibe_keyboard::domain::profile::{Action, ActionKind};

use super::super::action_form::action_fields;
use super::super::RED;

/// The hold action row: pick a kind, then its fields, then any problem. A hold runs while the key
/// is held past `[input] hold_ms`.
pub(super) fn hold_action_fields(ui: &mut egui::Ui, hold: &mut Option<Action>, page_count: usize) {
    let held_kind = hold.as_ref().map_or(ActionKind::None, Action::kind);
    let mut hold_kind = held_kind;
    egui::ComboBox::from_label("Hold action")
        .selected_text(hold_kind.label())
        .show_ui(ui, |ui| {
            for option in ActionKind::ALL {
                ui.selectable_value(&mut hold_kind, option, option.label());
            }
        });
    if hold_kind != held_kind {
        *hold = (hold_kind != ActionKind::None).then(|| Action::default_of(hold_kind));
    }
    if let Some(action) = hold.as_mut() {
        action_fields(ui, action, page_count);
    }
    if let Some(problem) = hold.as_ref().and_then(|action| action.problem(page_count)) {
        ui.colored_label(RED, problem);
    }
}
