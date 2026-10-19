//! Layout editor for `~/.config/vibe-keyboard/profile.toml`.
//! The running controller hot-reloads the file after you save.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui::{self, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Vec2};
use vibe_keyboard::domain::keys::is_nav_key;
use vibe_keyboard::domain::profile::{
    format_color, parse_color, Action, ActionKind, KeyConfig, Profile, BUILTIN_NAMES,
};
use vibe_keyboard::infrastructure::images::key_image;
use vibe_keyboard::infrastructure::profile_store;

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

/// A rendered key preview.
struct Preview {
    config: Option<KeyConfig>,
    texture: egui::TextureHandle,
}

struct Editor {
    root: PathBuf,
    profile: Profile,
    saved: Profile,
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
    /// Set when `profile.toml` could not be read. Saving is refused while it is set.
    load_error: Option<String>,
}

impl Editor {
    fn new() -> Self {
        let root = profile_store::root_dir();
        let (profile, load_error) =
            match profile_store::ensure_default(&profile_store::profile_path()) {
                Ok(profile) => (profile, None),
                Err(e) => (Profile::default_from_legacy(), Some(e)),
            };
        let mut editor = Self {
            root,
            saved: profile.clone(),
            profile,
            page: 0,
            key: None,
            status: "Loaded profile.".to_string(),
            previews: HashMap::new(),
            running: Vec::new(),
            running_rx: None,
            new_app: String::new(),
            load_error,
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
        self.profile != self.saved
    }

    fn save(&mut self) {
        if let Some(error) = &self.load_error {
            self.status =
                format!("Cannot save: {error}. Fix the file by hand, then reopen the editor.");
            return;
        }
        match profile_store::save(&profile_store::profile_path(), &self.profile) {
            Ok(()) => {
                self.saved = self.profile.clone();
                self.status = "Saved. The controller reloads within a second.".into();
            }
            Err(e) => self.status = format!("Save failed: {e}"),
        }
    }

    /// Texture of the deck image for `key`. Re-rendered only when that key's config changes.
    fn preview(&mut self, ctx: &egui::Context, key: u8) -> egui::TextureHandle {
        let config = self.profile.key_at(self.page, key).cloned();
        if let Some(cached) = self.previews.get(&key) {
            if cached.config == config {
                return cached.texture.clone();
            }
        }
        let img = key_image(key, config.as_ref(), &self.root);
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
            ui.weak(profile_store::profile_path().display().to_string());
        });
    }

    fn page_list(&mut self, ui: &mut egui::Ui) -> Option<PageOp> {
        let mut op = None;
        ui.heading("Pages");
        let count = self.profile.pages.len();
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
                    ui.text_edit_singleline(&mut self.profile.pages[index].name);
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
                        .add_enabled(count > 1, egui::Button::new("Delete"))
                        .clicked()
                    {
                        op = Some(PageOp::Delete(index));
                    }
                } else {
                    ui.label(self.profile.pages[index].name.as_str());
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

    /// Bundle ids that show the page at `index` when they are in front.
    fn app_rules(&mut self, ui: &mut egui::Ui, index: usize) {
        ui.label("Show this page when these apps are in front:");
        let mut remove = None;
        for (i, app) in self.profile.pages[index].apps.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.monospace(app);
                if ui.small_button("remove").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            self.profile.pages[index].apps.remove(i);
        }
        let mut add = None;
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_app);
            if ui.button("Add").clicked() {
                add = Some(self.new_app.trim().to_string());
            }
        });
        egui::ComboBox::from_id_salt("running-apps")
            .selected_text("Pick a running app")
            .show_ui(ui, |ui| {
                for (name, id) in &self.running {
                    if ui
                        .selectable_label(false, format!("{name} ({id})"))
                        .clicked()
                    {
                        add = Some(id.clone());
                    }
                }
            });
        if ui.button("Refresh running apps").clicked() {
            self.start_running_refresh();
        }
        if let Some(id) = add {
            let apps = &mut self.profile.pages[index].apps;
            if !id.is_empty() && !apps.contains(&id) {
                apps.push(id);
            }
            self.new_app.clear();
        }
    }

    fn apply_page_op(&mut self, op: PageOp) {
        match op {
            PageOp::Select(index) => {
                self.page = index;
                self.key = None;
            }
            PageOp::Add => {
                let name = format!("Page {}", self.profile.pages.len() + 1);
                self.page = self.profile.add_page(&name);
                self.key = None;
            }
            PageOp::Up(index) => {
                if self.profile.move_page(index, index.saturating_sub(1)) {
                    self.page = index - 1;
                }
            }
            PageOp::Down(index) => {
                if self.profile.move_page(index, index + 1) {
                    self.page = index + 1;
                }
            }
            PageOp::Delete(index) => {
                if self.profile.delete_page(index) {
                    self.page = self.page.min(self.profile.pages.len() - 1);
                    self.key = None;
                } else {
                    self.status = "A layout needs at least one page.".into();
                }
            }
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
        let Some(key) = self.key else {
            ui.label("Click a key to edit it.");
            return;
        };
        let page_count = self.profile.pages.len();
        let original = self
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
        ui.horizontal(|ui| {
            if ui.button("Browse image...").clicked() {
                let picked = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg"])
                    .pick_file();
                if let Some(path) = picked {
                    match profile_store::import_image(&self.root, &path) {
                        Ok(relative) => edited.image = Some(relative),
                        Err(e) => self.status = e,
                    }
                }
            }
            if edited.image.is_some() && ui.button("Clear image").clicked() {
                edited.image = None;
            }
        });
        if let Some(image) = &edited.image {
            ui.monospace(image.display().to_string());
        }

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

        ui.separator();
        if ui.button("Clear key").clicked() {
            clear = true;
        }

        // Write only real changes, so an untouched panel never dirties the profile.
        if clear {
            self.profile.clear_key(self.page, key);
        } else if edited != original {
            self.profile.set_key(self.page, key, edited);
        }
    }
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

/// Inputs for the fields of one action type.
fn action_fields(ui: &mut egui::Ui, action: &mut Action, page_count: usize) {
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
    }
}

impl eframe::App for Editor {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_running();
        egui::Panel::top("top").show(ui, |ui| self.top_bar(ui));
        egui::Panel::bottom("status").show(ui, |ui| {
            if let Some(error) = &self.load_error {
                ui.colored_label(RED, format!("profile.toml: {error}"));
            }
            ui.label(self.status.as_str());
            let problems = self.profile.problems();
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
            .show(ui, |ui| self.page_list(ui))
            .inner;
        egui::Panel::right("key")
            .resizable(true)
            .size_range(KEY_PANEL_WIDTH_RANGE)
            .default_size(KEY_PANEL_WIDTH)
            .show(ui, |ui| self.key_panel(ui));
        let clicked = egui::CentralPanel::default()
            .show(ui, |ui| self.grid(ui, &ctx))
            .inner;
        if let Some(op) = page_op {
            self.apply_page_op(op);
        }
        if let Some(key) = clicked {
            self.key = Some(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{cell_size, parse_lsappinfo, GAP, GRID_MARGIN, KEY_PANEL_WIDTH, PAGES_WIDTH};
    use eframe::egui::Vec2;

    #[test]
    fn grid_fits_the_default_and_minimum_window() {
        // Window minus both side panels and the top and bottom bars.
        let central =
            |window: Vec2| Vec2::new(window.x - PAGES_WIDTH - KEY_PANEL_WIDTH, window.y - 60.0);
        for window in [Vec2::new(1200.0, 720.0), Vec2::new(900.0, 600.0)] {
            let area = central(window);
            let cell = cell_size(area);
            let width = 5.0 * cell + 4.0 * GAP + GRID_MARGIN;
            let height = 3.0 * cell + 2.0 * GAP + GRID_MARGIN;
            assert!(width <= area.x + 0.5, "width at {window:?}");
            assert!(height <= area.y + 0.5, "height at {window:?}");
        }
    }

    #[test]
    fn parses_names_and_bundle_ids() {
        let text = "1) \"Finder\" ASN:0x0-0x1:\n    bundleID=\"com.apple.finder\"\n    executable=x\n2) \"Helper\" ASN:0x0-0x2:\n    bundleID=\"[ NULL ]\"\n";
        assert_eq!(
            parse_lsappinfo(text),
            vec![("Finder".to_string(), "com.apple.finder".to_string())]
        );
    }
}
