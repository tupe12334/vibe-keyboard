use eframe::egui;

use vibe_keyboard::domain::profile::Prompt;
use vibe_keyboard::domain::prompt_rules;

impl super::Editor {
    /// Edit the session prompts: label and text, reorder, delete, add. Changes are saved with the
    /// layout, so the same Save and Discard rules apply.
    pub(super) fn prompts_section(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.heading("Prompts");
        if self.session.profile.prompts.is_empty() {
            ui.label("None set. Session actions show no prompt keys.");
        }
        let count = self.session.profile.prompts.len();
        let mut op = None;
        for (index, prompt) in self.session.profile.prompts.iter_mut().enumerate() {
            ui.push_id(index, |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut prompt.label).desired_width(70.0));
                    ui.add(egui::TextEdit::singleline(&mut prompt.text).desired_width(120.0));
                });
                ui.horizontal(|ui| {
                    if ui.add_enabled(index > 0, egui::Button::new("Up")).clicked() {
                        op = Some(PromptOp::Up(index));
                    }
                    if ui
                        .add_enabled(index + 1 < count, egui::Button::new("Down"))
                        .clicked()
                    {
                        op = Some(PromptOp::Down(index));
                    }
                    if ui.button("Delete").clicked() {
                        op = Some(PromptOp::Delete(index));
                    }
                });
            });
        }
        if ui.button("Add prompt").clicked() {
            op = Some(PromptOp::Add);
        }
        if let Some(op) = op {
            apply_prompt_op(&mut self.session.profile.prompts, op);
        }
    }

    /// Timing of key holds: the `[input]` table of the profile.
    pub(super) fn input_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Input");
        ui.horizontal(|ui| {
            ui.label("Hold (ms)");
            ui.add(egui::DragValue::new(&mut self.session.profile.input.hold_ms).range(50..=5000));
        });
        ui.weak("A key held this long runs its hold action.");
    }
}

/// One edit to the prompt list. The index is the row the user pressed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PromptOp {
    Up(usize),
    Down(usize),
    Delete(usize),
    Add,
}

/// Apply `op` to the prompt list. The row rules live in the core crate.
pub(super) fn apply_prompt_op(prompts: &mut Vec<Prompt>, op: PromptOp) {
    match op {
        PromptOp::Up(index) => prompt_rules::move_prompt(prompts, index, false),
        PromptOp::Down(index) => prompt_rules::move_prompt(prompts, index, true),
        PromptOp::Delete(index) => prompt_rules::delete_prompt(prompts, index),
        PromptOp::Add => prompt_rules::add_prompt(prompts),
    }
}
