//! Layout editor for `~/.config/vibe-keyboard/profile.toml`.
//! The running controller hot-reloads the file after you save.

#[path = "editor/app.rs"]
mod app;
#[path = "editor/app_picker.rs"]
mod app_picker;
#[path = "editor/app_rules.rs"]
mod app_rules;
#[path = "editor/dialogs.rs"]
mod dialogs;
#[path = "editor/disk_actions.rs"]
mod disk_actions;
#[path = "editor/icon_grid.rs"]
mod icon_grid;
#[path = "editor/image_picker.rs"]
mod image_picker;
#[path = "editor/key_grid.rs"]
mod key_grid;
#[path = "editor/key_panel.rs"]
mod key_panel;
#[path = "editor/key_preview.rs"]
mod key_preview;
#[path = "editor/layout.rs"]
mod layout;
#[path = "editor/page_list.rs"]
mod page_list;
#[path = "editor/prompts.rs"]
mod prompts;
#[path = "editor/state.rs"]
mod state;
#[cfg(test)]
use layout::GRID_MARGIN;
use state::{Editor, PageOp};
#[path = "editor/running_apps.rs"]
mod running_apps;
#[path = "editor/status_bar.rs"]
mod status_bar;
use layout::{
    cell_size, side_strip_size, GAP, GRID, KEY_PANEL_WIDTH, KEY_PANEL_WIDTH_RANGE, PAGES_WIDTH,
    PAGES_WIDTH_RANGE, RED,
};
#[cfg(test)]
use prompts::{apply_prompt_op, PromptOp};
#[cfg(test)]
use running_apps::parse_lsappinfo;
#[path = "editor/side_panel.rs"]
mod side_panel;
#[path = "editor/side_strip.rs"]
mod side_strip;

fn main() -> eframe::Result {
    eframe::run_native(
        "vibe-keyboard editor",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 720.0])
                .with_min_inner_size([900.0, 600.0]),
            ..Default::default()
        },
        Box::new(|_cc| {
            let app: Box<dyn eframe::App> = Box::new(Editor::new());
            Ok(app)
        }),
    )
}

#[path = "editor/image_kind.rs"]
mod image_kind;
#[path = "editor/previews.rs"]
mod previews;
#[path = "editor/saving.rs"]
mod saving;
use image_kind::ImageKind;
use previews::Preview;

#[path = "editor/session.rs"]
mod session;
use session::load_from_disk;
#[cfg(test)]
use session::Session;
#[cfg(test)]
use session::UNDO_WINDOW;

#[path = "editor/action_form.rs"]
mod action_form;
#[path = "editor/widgets.rs"]
mod widgets;

#[cfg(test)]
#[path = "editor/tests.rs"]
mod tests;
