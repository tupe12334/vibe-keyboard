//! Layout editor for `~/.config/vibe-keyboard/profile.toml`.
//! The running controller hot-reloads the file after you save.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, Vec2};
use vibe_keyboard::domain::profile_rules;
use vibe_keyboard::infrastructure::images::{
    app_icon_generation, key_image, SIDE_HEIGHT, SIDE_WIDTH,
};
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
use previews::{Preview, SidePreview};

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
}

#[path = "editor/action_form.rs"]
mod action_form;
#[path = "editor/widgets.rs"]
mod widgets;

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
