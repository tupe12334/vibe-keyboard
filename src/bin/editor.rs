//! Layout editor for `~/.config/vibe-keyboard/profile.toml`.
//! The running controller hot-reloads the file after you save.

use eframe::egui::Color32;
use vibe_keyboard::infrastructure::images::SIDE_HEIGHT;
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
use state::Editor;
#[path = "editor/running_apps.rs"]
mod running_apps;
#[path = "editor/status_bar.rs"]
mod status_bar;
use layout::{cell_size, side_strip_size};
#[cfg(test)]
use prompts::{apply_prompt_op, PromptOp};
#[cfg(test)]
use running_apps::parse_lsappinfo;
#[path = "editor/side_panel.rs"]
mod side_panel;
#[path = "editor/side_strip.rs"]
mod side_strip;

/// Physical layout as seen from the front. Matches `device.md`.
const GRID: [[u8; 5]; 3] = [[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15]];
/// Grid cell size in points. The deck key is square.
const CELL_MAX: f32 = 96.0;
const CELL_MIN: f32 = 24.0;
/// Gap between grid cells, and the margin kept free around the grid.
const GAP: f32 = 8.0;
const GRID_MARGIN: f32 = 16.0;
const PAGES_WIDTH: f32 = 260.0;
const PAGES_WIDTH_RANGE: std::ops::RangeInclusive<f32> = 200.0..=360.0;
const KEY_PANEL_WIDTH: f32 = 320.0;
const KEY_PANEL_WIDTH_RANGE: std::ops::RangeInclusive<f32> = 260.0..=420.0;
const RED: Color32 = Color32::from_rgb(230, 80, 80);
/// Side strip preview: at most the device render height, same 100:480 ratio.
const SIDE_STRIP_MAX: f32 = SIDE_HEIGHT as f32;

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

/// A page-list change, applied after the sidebar is drawn.
enum PageOp {
    Select(usize),
    Add,
    Up(usize),
    Down(usize),
    Delete(usize),
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
