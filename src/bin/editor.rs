//! Layout editor for `~/.config/vibe-keyboard/profile.toml`.
//! The running controller hot-reloads the file after you save.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Vec2};
use vibe_keyboard::domain::profile::{format_color, inner_rect, parse_color, Action, ActionKind};
use vibe_keyboard::domain::{keys::is_nav_key, profile_rules};
use vibe_keyboard::infrastructure::images::{
    app_icon_generation, key_image, SIDE_HEIGHT, SIDE_WIDTH,
};
#[path = "editor/app_picker.rs"]
mod app_picker;
#[path = "editor/app_rules.rs"]
mod app_rules;
#[path = "editor/icon_grid.rs"]
mod icon_grid;
#[path = "editor/image_picker.rs"]
mod image_picker;
#[path = "editor/page_list.rs"]
mod page_list;
#[path = "editor/prompts.rs"]
mod prompts;
#[cfg(test)]
use prompts::{apply_prompt_op, PromptOp};
#[path = "editor/side_panel.rs"]
mod side_panel;
#[path = "editor/side_strip.rs"]
mod side_strip;
use vibe_keyboard::infrastructure::profile_store;
use vibe_keyboard::infrastructure::system_metrics::{self, Shared};
use vibe_keyboard::infrastructure::usb::DECK_IMAGE_SIZE;

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
use previews::{shade_hidden, Preview, SidePreview};

#[path = "editor/session.rs"]
mod session;
#[cfg(test)]
use session::UNDO_WINDOW;
use session::{load_from_disk, Session};

struct Editor {
    root: PathBuf,
    session: Session,
    /// The unsaved-changes dialog is open (window close was requested while dirty).
    close_prompt: bool,
    /// File modification time when it was last loaded or written by this editor.
    loaded_mtime: Option<std::time::SystemTime>,
    /// The file changed on disk since it was loaded. Save waits for the user's choice.
    disk_conflict: bool,
    page: usize,
    key: Option<u8>,
    status: String,
    /// Last rendered preview per key, with the config it was rendered from.
    previews: HashMap<u8, Preview>,
    /// Running apps as (name, bundle id), refreshed on demand.
    running: Vec<(String, String)>,
    /// Pending result of a background `lsappinfo` run.
    running_rx: Option<mpsc::Receiver<Vec<(String, String)>>>,
    new_app: String,
    /// Bundle id typed into the app image picker.
    image_app_text: String,
    /// Textures of the icon pack, loaded on first use.
    icon_textures: HashMap<&'static str, egui::TextureHandle>,
    /// Set when `profile.toml` could not be read. Saving is refused while it is set.
    load_error: Option<String>,
    /// Latest system sample, shared with the sampler thread. Widgets in the preview use it.
    metrics: Shared,
    /// Last rendered side preview of the current page.
    side_preview: Option<SidePreview>,
    /// The side panel is shown instead of the key panel.
    side_open: bool,
}

impl Editor {
    fn new() -> Self {
        let root = profile_store::root_dir();
        let (profile, load_error) = load_from_disk();
        let mut editor = Self {
            root,
            session: Session::new(profile),
            close_prompt: false,
            loaded_mtime: profile_store::modified_at(&profile_store::profile_path()),
            disk_conflict: false,
            page: 0,
            key: None,
            status: "Loaded profile.".to_string(),
            previews: HashMap::new(),
            running: Vec::new(),
            running_rx: None,
            new_app: String::new(),
            image_app_text: String::new(),
            icon_textures: HashMap::new(),
            load_error,
            side_preview: None,
            side_open: false,
            metrics: system_metrics::start(),
        };
        editor.start_running_refresh();
        editor
    }

    /// Run `lsappinfo` on a background thread. `ui` picks up the result.
    fn start_running_refresh(&mut self) {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(running_apps());
        });
        self.running_rx = Some(rx);
    }

    fn poll_running(&mut self) {
        if let Some(rx) = &self.running_rx {
            if let Ok(apps) = rx.try_recv() {
                self.running = apps;
                self.running_rx = None;
            }
        }
    }

    fn dirty(&self) -> bool {
        self.session.profile != self.session.saved
    }

    /// Write the layout to disk. Returns true when the file was written.
    fn save(&mut self) -> bool {
        self.write_file(false)
    }

    /// Write the layout. Refuses on load errors, layout problems, or (unless `overwrite`)
    /// when the file changed on disk since it was loaded.
    fn write_file(&mut self, overwrite: bool) -> bool {
        if let Some(error) = &self.load_error {
            self.status =
                format!("Cannot save: {error}. Fix the file by hand, then reopen the editor.");
            return false;
        }
        match saving::write(&self.session.profile, self.loaded_mtime, overwrite) {
            saving::Outcome::Saved => {
                self.session.mark_saved();
                self.loaded_mtime = profile_store::modified_at(&profile_store::profile_path());
                self.disk_conflict = false;
                self.status = "Saved. The controller reloads within a second.".into();
                true
            }
            saving::Outcome::Conflict => {
                self.disk_conflict = true;
                false
            }
            saving::Outcome::Refused(message) => {
                self.status = message;
                false
            }
        }
    }

    /// Adopt the file as it is on disk now, the same way startup loads it.
    fn adopt_disk(&mut self) {
        let (disk, error) = load_from_disk();
        self.load_error = error;
        self.session.replace_with_disk(disk);
        self.loaded_mtime = profile_store::modified_at(&profile_store::profile_path());
        self.disk_conflict = false;
        self.key = None;
        self.previews.clear();
        self.clamp_selection();
    }

    /// Drop unsaved edits and reload the file, the same way startup does.
    fn discard(&mut self) {
        if !self.session.dirty() {
            return;
        }
        let (disk, error) = load_from_disk();
        if self.session.discard(disk, Instant::now()) {
            self.load_error = error;
            self.loaded_mtime = profile_store::modified_at(&profile_store::profile_path());
            self.key = None;
            self.previews.clear();
            self.clamp_selection();
            self.side_preview = None;
            self.status = "Changes discarded".into();
        }
    }

    fn undo(&mut self) {
        if self.session.undo(Instant::now()) {
            self.previews.clear();
            self.clamp_selection();
            self.side_preview = None;
            self.status = "Discard undone".into();
        }
    }

    /// Keep the selected page inside the current layout. The key selection is dropped when
    /// the page moves.
    fn clamp_selection(&mut self) {
        let count = self.session.profile.pages.len();
        let page = self.page.min(count.saturating_sub(1));
        if page != self.page {
            self.page = page;
            self.key = None;
        }
    }

    /// Shown when the file changed on disk since it was loaded.
    fn conflict_dialog(&mut self, ctx: &egui::Context) {
        let (mut reload, mut overwrite, mut keep) = (false, false, false);
        egui::Window::new("profile.toml changed on disk")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Another program changed the file after it was loaded.");
                ui.label("Reload discards your edits. Overwrite replaces the file.");
                ui.horizontal(|ui| {
                    reload |= ui.button("Reload").clicked();
                    overwrite |= ui.button("Overwrite").clicked();
                    keep |= ui.button("Keep editing").clicked();
                });
            });
        if reload {
            self.adopt_disk();
            self.status = "Reloaded profile.toml from disk.".into();
        } else if overwrite {
            self.write_file(true);
        } else if keep {
            self.disk_conflict = false;
        }
    }

    /// Texture of the deck image for `key`. Re-rendered only when that key's config changes.
    fn preview(&mut self, ctx: &egui::Context, key: u8) -> egui::TextureHandle {
        let config = self.session.profile.key_at(self.page, key).cloned();
        let inset = self.session.profile.display.key_inset;
        let generation = app_icon_generation();
        if let Some(cached) = self.previews.get(&key) {
            if cached.config == config && cached.generation == generation && cached.inset == inset {
                return cached.texture.clone();
            }
        }
        let img = key_image(key, config.as_ref(), &self.root, &inset);
        let size = [img.width() as usize, img.height() as usize];
        let rgba: Vec<u8> = img
            .pixels()
            .flat_map(|p| [p.0[0], p.0[1], p.0[2], 255])
            .collect();
        let color = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
        let texture = ctx.load_texture(format!("key-{key}"), color, egui::TextureOptions::LINEAR);
        self.previews.insert(
            key,
            Preview {
                config,
                inset,
                generation,
                texture: texture.clone(),
            },
        );
        texture
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Stream deck layout");
            if self.dirty() {
                ui.colored_label(Color32::YELLOW, "Unsaved changes");
            }
            let dirty = self.dirty();
            let can_save = dirty && self.load_error.is_none();
            if ui
                .add_enabled(can_save, egui::Button::new("Save"))
                .clicked()
            {
                self.save();
            }
            if ui
                .add_enabled(dirty, egui::Button::new("Discard changes"))
                .clicked()
            {
                self.discard();
            }
            ui.weak(profile_store::profile_path().display().to_string());
        });
    }

    /// Modal shown when the window closes with unsaved edits. Cancel is the default.
    fn close_dialog(&mut self, ctx: &egui::Context) {
        let (escape, enter) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Enter),
            )
        });
        let (mut save, mut discard, mut cancel) = (false, false, escape || enter);
        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Save changes to profile.toml before closing?");
                ui.horizontal(|ui| {
                    save |= ui
                        .add_enabled(self.load_error.is_none(), egui::Button::new("Save"))
                        .clicked();
                    discard |= ui.button("Discard").clicked();
                    cancel |= ui.button("Cancel").clicked();
                });
            });
        if cancel {
            self.close_prompt = false;
        } else if discard || (save && self.save()) {
            self.close_prompt = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn grid(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> Option<u8> {
        let mut clicked = None;
        let cell = cell_size(ui.available_size());
        egui::Grid::new("keys").spacing([GAP, GAP]).show(ui, |ui| {
            for row in GRID {
                for key in row {
                    let response = self.key_cell(ui, ctx, key, cell);
                    if response.clicked() && !is_nav_key(key) {
                        clicked = Some(key);
                    }
                }
                ui.end_row();
            }
        });
        clicked
    }

    fn key_cell(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        key: u8,
        cell: f32,
    ) -> egui::Response {
        let size = Vec2::splat(cell);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let texture = self.preview(ctx, key);
        let painter = ui.painter_at(rect);
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(texture.id(), rect, uv, Color32::WHITE);
        let inset = self.session.profile.display.key_inset;
        let visible = inner_rect(
            DECK_IMAGE_SIZE,
            DECK_IMAGE_SIZE,
            inset.top,
            inset.bottom,
            inset.left,
            inset.right,
        );
        shade_hidden(&painter, rect, (DECK_IMAGE_SIZE, DECK_IMAGE_SIZE), visible);
        // Key number in a small dark box, so it reads over any image.
        let number =
            egui::Rect::from_min_size(rect.left_top() + Vec2::new(2.0, 2.0), Vec2::new(18.0, 14.0));
        painter.rect_filled(number, 3.0, Color32::from_black_alpha(160));
        painter.text(
            number.center(),
            Align2::CENTER_CENTER,
            key.to_string(),
            FontId::monospace(10.0),
            Color32::WHITE,
        );
        if self.key == Some(key) {
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(2.0, Color32::LIGHT_BLUE),
                StrokeKind::Inside,
            );
        }
        response
    }

    fn key_panel(&mut self, ui: &mut egui::Ui) {
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

#[path = "editor/action_form.rs"]
mod action_form;
#[path = "editor/widgets.rs"]
mod widgets;
use action_form::action_fields;

/// Size of the side strip for `avail_height`: at most the device render height, 100:480 ratio.
fn side_strip_size(avail_height: f32) -> Vec2 {
    let height = (avail_height - GRID_MARGIN).clamp(1.0, SIDE_STRIP_MAX);
    Vec2::new(height * SIDE_WIDTH as f32 / SIDE_HEIGHT as f32, height)
}

/// Largest square cell that fits the 3x5 grid in `avail`, never below `CELL_MIN`.
fn cell_size(avail: Vec2) -> f32 {
    let by_width = (avail.x - GRID_MARGIN - 4.0 * GAP) / 5.0;
    let by_height = (avail.y - GRID_MARGIN - 2.0 * GAP) / 3.0;
    by_width.min(by_height).clamp(CELL_MIN, CELL_MAX)
}

/// Running apps as (name, bundle id), from `lsappinfo list`. Empty if it fails.
fn running_apps() -> Vec<(String, String)> {
    std::process::Command::new("lsappinfo")
        .arg("list")
        .output()
        .map(|out| parse_lsappinfo(&String::from_utf8_lossy(&out.stdout)))
        .unwrap_or_default()
}

/// Parse `lsappinfo list`. Each app starts with `N) "Name"` and has a `bundleID="..."` line.
fn parse_lsappinfo(text: &str) -> Vec<(String, String)> {
    let mut apps = Vec::new();
    let mut name: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some((number, rest)) = trimmed.split_once(") \"") {
            if number.chars().all(|c| c.is_ascii_digit()) {
                name = rest.split('"').next().map(str::to_string);
            }
        } else if let Some(rest) = trimmed.strip_prefix("bundleID=\"") {
            if let (Some(app), Some(id)) = (name.take(), rest.split('"').next()) {
                if id != "[ NULL ]" && !id.is_empty() {
                    apps.push((app, id.to_string()));
                }
            }
        }
    }
    apps
}

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
        egui::Panel::bottom("status").show(ui, |ui| {
            if let Some(error) = &self.load_error {
                ui.colored_label(RED, format!("profile.toml: {error}"));
            }
            if self.session.undo_available(now) {
                ui.horizontal(|ui| {
                    ui.label("Changes discarded —");
                    if ui.button("Undo").clicked() {
                        self.undo();
                    }
                });
            } else {
                ui.label(self.status.as_str());
            }
            let problems = profile_rules::save_problems(&self.session.profile);
            if !problems.is_empty() {
                ui.collapsing(
                    format!("{} problem(s) in the layout", problems.len()),
                    |ui| {
                        for problem in &problems {
                            ui.colored_label(RED, problem);
                        }
                    },
                );
            }
        });
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

#[cfg(test)]
#[path = "editor/tests.rs"]
mod tests;
