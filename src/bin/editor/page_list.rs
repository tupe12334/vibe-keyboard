use eframe::egui;

use super::PageOp;
use vibe_keyboard::domain::profile_rules;

impl super::Editor {
    pub(super) fn page_list(&mut self, ui: &mut egui::Ui) -> Option<PageOp> {
        let mut op = None;
        ui.heading("Pages");
        let count = self.session.profile.pages.len();
        for index in 0..count {
            let selected = index == self.page;
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(selected, format!("{}", index + 1))
                    .clicked()
                {
                    op = Some(PageOp::Select(index));
                }
                if selected {
                    ui.text_edit_singleline(&mut self.session.profile.pages[index].name);
                    if ui.add_enabled(index > 0, egui::Button::new("Up")).clicked() {
                        op = Some(PageOp::Up(index));
                    }
                    if ui
                        .add_enabled(index + 1 < count, egui::Button::new("Down"))
                        .clicked()
                    {
                        op = Some(PageOp::Down(index));
                    }
                    if ui
                        .add_enabled(
                            profile_rules::can_delete_page(&self.session.profile),
                            egui::Button::new("Delete"),
                        )
                        .clicked()
                    {
                        op = Some(PageOp::Delete(index));
                    }
                } else {
                    ui.label(self.session.profile.pages[index].name.as_str());
                }
            });
            if selected {
                self.app_rules(ui, index);
            }
        }
        ui.separator();
        if ui.button("Add page").clicked() {
            op = Some(PageOp::Add);
        }
        op
    }

    pub(super) fn apply_page_op(&mut self, op: &PageOp) {
        match *op {
            PageOp::Select(index) => {
                self.page = index;
                self.key = None;
            }
            PageOp::Add => {
                self.page = profile_rules::add_page(&mut self.session.profile);
                self.key = None;
            }
            PageOp::Up(index) => {
                if self
                    .session
                    .profile
                    .move_page(index, index.saturating_sub(1))
                {
                    self.page = index - 1;
                }
            }
            PageOp::Down(index) => {
                if self.session.profile.move_page(index, index + 1) {
                    self.page = index + 1;
                }
            }
            PageOp::Delete(index) => {
                if self.session.profile.delete_page(index) {
                    self.page = self.page.min(self.session.profile.pages.len() - 1);
                    self.key = None;
                } else {
                    self.status = "A layout needs at least one page.".into();
                }
            }
        }
    }
}
