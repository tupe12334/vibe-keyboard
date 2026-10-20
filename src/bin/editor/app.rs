use std::time::{Duration, Instant};

use eframe::egui;

use super::Editor;
use super::{KEY_PANEL_WIDTH, KEY_PANEL_WIDTH_RANGE, PAGES_WIDTH, PAGES_WIDTH_RANGE};

impl eframe::App for Editor {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_running();
        let now = Instant::now();
        self.session.expire(now);
        let (save_key, undo_key, close) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::S),
                i.modifiers.command && i.key_pressed(egui::Key::Z),
                i.viewport().close_requested(),
            )
        });
        if save_key && self.session.dirty() {
            self.save();
        }
        // Cmd+Z undoes only the last discard. Text fields keep their own undo.
        if undo_key && !ctx.egui_wants_keyboard_input() && self.session.undo_available(now) {
            self.undo();
        }
        if close && self.session.dirty() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_prompt = true;
        }
        if self.close_prompt {
            self.close_dialog(&ctx);
        }
        if self.disk_conflict {
            self.conflict_dialog(&ctx);
        }
        if self.session.undo_available(now) {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
        egui::Panel::top("top").show(ui, |ui| self.top_bar(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui, now));
        let page_op = egui::Panel::left("pages")
            .resizable(true)
            .size_range(PAGES_WIDTH_RANGE)
            .default_size(PAGES_WIDTH)
            .show(ui, |ui| {
                let op = self.page_list(ui);
                self.prompts_section(ui);
                ui.separator();
                self.display_section(ui);
                self.input_section(ui);
                op
            })
            .inner;
        egui::Panel::right("key")
            .resizable(true)
            .size_range(KEY_PANEL_WIDTH_RANGE)
            .default_size(KEY_PANEL_WIDTH)
            .show(ui, |ui| self.key_panel(ui));
        let (side_clicked, clicked) = egui::CentralPanel::default()
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let side = self.side_strip(ui, &ctx);
                    (side, self.grid(ui, &ctx))
                })
                .inner
            })
            .inner;
        if let Some(op) = page_op {
            self.apply_page_op(&op);
        }
        if side_clicked {
            self.side_open = true;
            self.key = None;
        }
        if let Some(key) = clicked {
            self.key = Some(key);
            self.side_open = false;
        }
    }
}
