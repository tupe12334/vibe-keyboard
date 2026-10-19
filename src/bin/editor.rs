//! Layout editor for `~/.config/vibe-keyboard/profile.toml`.
//! The running controller hot-reloads the file after you save.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Vec2};
use vibe_keyboard::domain::keys::is_nav_key;
use vibe_keyboard::domain::profile::{
    format_color, parse_color, Action, ActionKind, KeyConfig, Profile, Prompt, SideConfig, Widget,
    BUILTIN_NAMES,
};
use vibe_keyboard::infrastructure::images::{
    app_icon_generation, icon_image, key_image, side_image, ICON_NAMES, SAFE_WIDTH, SAFE_X,
    SIDE_HEIGHT, SIDE_WIDTH,
};
use vibe_keyboard::infrastructure::profile_store;
use vibe_keyboard::infrastructure::system_metrics::{self, Metrics, Shared};

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
/// Side strip preview: at most the device width, same 480:100 ratio.
const SIDE_STRIP_MAX: f32 = SIDE_WIDTH as f32;

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

/// Where a key's image comes from, read from the stored text.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ImageKind {
    None,
    File,
    Icon,
    App,
}

impl ImageKind {
    const ALL: [Self; 4] = [Self::None, Self::File, Self::Icon, Self::App];

    fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::File => "File",
            Self::Icon => "Icon",
            Self::App => "App icon",
        }
    }

    fn of(image: Option<&std::path::Path>) -> Self {
        let Some(path) = image else {
            return Self::None;
        };
        let text = path.to_string_lossy();
        if text.starts_with("icon:") {
            Self::Icon
        } else if text.starts_with("app:") {
            Self::App
        } else {
            Self::File
        }
    }
}

/// Upload an RGB image as an egui texture, fully opaque.
fn texture_from(ctx: &egui::Context, name: &str, img: &image::RgbImage) -> egui::TextureHandle {
    let size = [img.width() as usize, img.height() as usize];
    let rgba: Vec<u8> = img
        .pixels()
        .flat_map(|p| [p.0[0], p.0[1], p.0[2], 255])
        .collect();
    let color = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    ctx.load_texture(name.to_string(), color, egui::TextureOptions::LINEAR)
}

/// A rendered key preview.
struct Preview {
    config: Option<KeyConfig>,
    /// `app_icon_generation()` when rendered, so finished app lookups show up.
    generation: u64,
    texture: egui::TextureHandle,
}

/// Rendered side preview, with the config it was rendered from.
struct SidePreview {
    config: Option<SideConfig>,
    metrics: Metrics,
    texture: egui::TextureHandle,
}

/// How long the Undo for a discard stays available.
const UNDO_WINDOW: Duration = Duration::from_secs(10);

/// Edits in memory and the last file state. Pure, so its transitions are unit tested.
#[derive(Default)]
struct Session {
    profile: Profile,
    /// The profile as last read from or written to disk.
    saved: Profile,
    /// The edits dropped by the last discard, and when they were dropped.
    dropped: Option<(Profile, Instant)>,
}

impl Session {
    fn new(profile: Profile) -> Self {
        Self {
            saved: profile.clone(),
            profile,
            dropped: None,
        }
    }

    fn dirty(&self) -> bool {
        self.profile != self.saved
    }

    /// A save wrote `profile` to disk. Earlier discards can no longer be undone.
    fn mark_saved(&mut self) {
        self.saved = self.profile.clone();
        self.dropped = None;
    }

    /// Drop unsaved edits and adopt `disk`, the file as it is now. False when nothing is dirty.
    /// Replace everything with the file as it is on disk. Used when the user chooses Reload.
    fn replace_with_disk(&mut self, disk: Profile) {
        self.saved = disk.clone();
        self.profile = disk;
        self.dropped = None;
    }

    fn discard(&mut self, disk: Profile, now: Instant) -> bool {
        if !self.dirty() {
            return false;
        }
        let dropped = std::mem::replace(&mut self.profile, disk.clone());
        self.dropped = Some((dropped, now));
        self.saved = disk;
        true
    }

    fn undo_available(&self, now: Instant) -> bool {
        self.dropped
            .as_ref()
            .is_some_and(|(_, at)| now.duration_since(*at) <= UNDO_WINDOW)
    }

    /// Restore the edits dropped by the last discard, if still in the undo window.
    fn undo(&mut self, now: Instant) -> bool {
        if !self.undo_available(now) {
            self.dropped = None;
            return false;
        }
        if let Some((profile, _)) = self.dropped.take() {
            self.profile = profile;
        }
        true
    }

    fn expire(&mut self, now: Instant) {
        if !self.undo_available(now) {
            self.dropped = None;
        }
    }
}

/// Read the profile the way startup does. Returns the layout and the read error, if any.
fn load_from_disk() -> (Profile, Option<String>) {
    match profile_store::ensure_default(&profile_store::profile_path()) {
        Ok(profile) => (profile, None),
        Err(e) => (Profile::default_from_legacy(), Some(e)),
    }
}

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
        let problems = self.session.profile.problems();
        if !problems.is_empty() {
            self.status = format!(
                "Cannot save: {} problem(s) in the layout. Fix them first (listed below).",
                problems.len()
            );
            return false;
        }
        let path = profile_store::profile_path();
        if !overwrite && profile_store::modified_at(&path) != self.loaded_mtime {
            self.disk_conflict = true;
            return false;
        }
        match profile_store::save(&path, &self.session.profile) {
            Ok(()) => {
                self.session.mark_saved();
                self.loaded_mtime = profile_store::modified_at(&path);
                self.disk_conflict = false;
                self.status = "Saved. The controller reloads within a second.".into();
                true
            }
            Err(e) => {
                self.status = format!("Save failed: {e}");
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
        let generation = app_icon_generation();
        if let Some(cached) = self.previews.get(&key) {
            if cached.config == config && cached.generation == generation {
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
                generation,
                texture: texture.clone(),
            },
        );
        texture
    }

    /// Texture of the side screen of the current page. Re-rendered only when its config changes.
    /// Re-rendered when its config or the latest system sample changes. Widget pages ask for
    /// a repaint every sample, so the values stay live.
    fn side_texture(&mut self, ctx: &egui::Context) -> egui::TextureHandle {
        let config = self
            .session
            .profile
            .pages
            .get(self.page)
            .and_then(|p| p.side.clone());
        let metrics = system_metrics::snapshot(&self.metrics);
        if config.as_ref().is_some_and(|c| !c.widgets.is_empty()) {
            ctx.request_repaint_after(system_metrics::SAMPLE_INTERVAL);
        }
        if let Some(cached) = &self.side_preview {
            if cached.config == config && cached.metrics == metrics {
                return cached.texture.clone();
            }
        }
        let img = side_image(config.as_ref(), &self.root, &metrics);
        let texture = texture_from(ctx, "side", &img);
        self.side_preview = Some(SidePreview {
            config,
            metrics,
            texture: texture.clone(),
        });
        texture
    }

    /// Side preview strip above the keys, with the safe area drawn faintly. True when clicked.
    fn side_strip(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let size = side_strip_size(ui.available_width());
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let texture = self.side_texture(ctx);
        let painter = ui.painter_at(rect);
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(texture.id(), rect, uv, Color32::WHITE);
        let scale = size.x / SIDE_WIDTH as f32;
        let safe = Rect::from_min_size(
            rect.min + Vec2::new(SAFE_X as f32 * scale, 0.0),
            Vec2::new(SAFE_WIDTH as f32 * scale, size.y),
        );
        painter.rect_stroke(
            safe,
            0.0,
            Stroke::new(1.0, Color32::from_white_alpha(70)),
            StrokeKind::Inside,
        );
        if self.side_open {
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(2.0, Color32::LIGHT_BLUE),
                StrokeKind::Inside,
            );
        }
        ui.add_space(GAP);
        response.clicked()
    }

    /// Edit the side screen of the current page: label, colors, image. Add or remove it.
    fn side_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Side screen");
        let page = self.page;
        let Some(original) = self
            .session
            .profile
            .pages
            .get(page)
            .and_then(|p| p.side.clone())
        else {
            ui.label("This page has no side screen.");
            if ui.button("Add side screen").clicked() {
                self.session
                    .profile
                    .set_side(page, Some(SideConfig::default()));
            }
            return;
        };
        let mut edited = original.clone();
        let ctx = ui.ctx().clone();
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
        self.image_picker(ui, &ctx, &mut edited.image);
        widget_editor(ui, &mut edited.widgets);
        ui.separator();
        let remove = ui.button("Remove side screen").clicked();
        // Write only real changes, so an untouched panel never dirties the profile.
        if remove {
            self.session.profile.set_side(page, None);
        } else if edited != original {
            self.session.profile.set_side(page, Some(edited));
        }
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

    /// Edit the session prompts: label and text, reorder, delete, add. Changes are saved with the
    /// layout, so the same Save and Discard rules apply.
    fn prompts_section(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.heading("Prompts");
        if self.session.profile.prompts.is_empty() {
            ui.label("None set. Session actions use Recap and Voice.");
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

    fn page_list(&mut self, ui: &mut egui::Ui) -> Option<PageOp> {
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
                        .add_enabled(count > 1, egui::Button::new("Delete"))
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

    /// Image source for one key: none, a file, an embedded icon, or an installed app's icon.
    fn image_picker(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        image: &mut Option<PathBuf>,
    ) {
        let current = ImageKind::of(image.as_deref());
        let mut chosen = current;
        egui::ComboBox::from_label("Image")
            .selected_text(current.label())
            .show_ui(ui, |ui| {
                for kind in ImageKind::ALL {
                    ui.selectable_value(&mut chosen, kind, kind.label());
                }
            });
        if chosen != current {
            *image = match chosen {
                ImageKind::None => None,
                // Empty placeholders: the key shows its label until a value is chosen.
                ImageKind::File => Some(PathBuf::new()),
                ImageKind::Icon => Some(PathBuf::from("icon:back")),
                ImageKind::App => Some(PathBuf::from("app:")),
            };
        }
        match ImageKind::of(image.as_deref()) {
            ImageKind::None => {}
            ImageKind::File => {
                ui.horizontal(|ui| {
                    if ui.button("Browse...").clicked() {
                        let picked = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg"])
                            .pick_file();
                        if let Some(path) = picked {
                            match profile_store::import_image(&self.root, &path) {
                                Ok(relative) => *image = Some(relative),
                                Err(e) => self.status = e,
                            }
                        }
                    }
                    if let Some(path) = image.as_ref() {
                        ui.monospace(path.display().to_string());
                    }
                });
            }
            ImageKind::Icon => self.icon_grid(ui, ctx, image),
            ImageKind::App => self.app_picker(ui, image),
        }
    }

    /// Icon pack as clickable thumbnails. The selected icon is highlighted.
    fn icon_grid(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, image: &mut Option<PathBuf>) {
        let selected = image
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            for name in ICON_NAMES.iter().copied() {
                let Some(texture) = self.icon_texture(ctx, name) else {
                    continue;
                };
                let thumb = egui::Image::new((texture.id(), Vec2::splat(40.0)));
                let button =
                    egui::Button::image(thumb).selected(selected == format!("icon:{name}"));
                if ui.add(button).on_hover_text(name).clicked() {
                    *image = Some(PathBuf::from(format!("icon:{name}")));
                }
            }
        });
    }

    /// Installed-app icon by bundle id: pick a running app, or type the id.
    fn app_picker(&mut self, ui: &mut egui::Ui, image: &mut Option<PathBuf>) {
        let bundle = image
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let bundle = bundle.strip_prefix("app:").unwrap_or_default().to_string();
        ui.monospace(if bundle.is_empty() {
            "(no app chosen)".to_string()
        } else {
            bundle.clone()
        });
        let mut pick = None;
        egui::ComboBox::from_id_salt("image-app")
            .selected_text("Pick a running app")
            .show_ui(ui, |ui| {
                for (name, id) in &self.running {
                    if ui
                        .selectable_label(false, format!("{name} ({id})"))
                        .clicked()
                    {
                        pick = Some(id.clone());
                    }
                }
            });
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.image_app_text);
            if ui.button("Use").clicked() {
                pick = Some(self.image_app_text.trim().to_string());
            }
        });
        if let Some(id) = pick.filter(|id| !id.is_empty()) {
            *image = Some(PathBuf::from(format!("app:{id}")));
        }
    }

    /// Texture of one icon-pack image, cached per name.
    fn icon_texture(
        &mut self,
        ctx: &egui::Context,
        name: &'static str,
    ) -> Option<egui::TextureHandle> {
        if let Some(texture) = self.icon_textures.get(name) {
            return Some(texture.clone());
        }
        let img = icon_image(name, 64)?;
        let texture = texture_from(ctx, name, &img);
        self.icon_textures.insert(name, texture.clone());
        Some(texture)
    }

    /// Bundle ids that show the page at `index` when they are in front.
    fn app_rules(&mut self, ui: &mut egui::Ui, index: usize) {
        ui.label("Show this page when these apps are in front:");
        let mut remove = None;
        for (i, app) in self.session.profile.pages[index].apps.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.monospace(app);
                if ui.small_button("remove").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            self.session.profile.pages[index].apps.remove(i);
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
            let apps = &mut self.session.profile.pages[index].apps;
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
                let name = format!("Page {}", self.session.profile.pages.len() + 1);
                self.page = self.session.profile.add_page(&name);
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

/// List of side widgets: reorder, remove, edit text, and add one of each kind.
/// Widgets replace the side label and image when the list is not empty.
fn widget_editor(ui: &mut egui::Ui, widgets: &mut Vec<Widget>) {
    ui.separator();
    ui.strong("Widgets");
    let last = widgets.len().saturating_sub(1);
    let (mut remove, mut swap) = (None, None);
    for (i, widget) in widgets.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(widget_name(widget));
            if let Widget::Text { value } = widget {
                ui.text_edit_singleline(value);
            }
            if ui.add_enabled(i > 0, egui::Button::new("Up")).clicked() {
                swap = Some((i, i - 1));
            }
            if ui
                .add_enabled(i < last, egui::Button::new("Down"))
                .clicked()
            {
                swap = Some((i, i + 1));
            }
            if ui.button("Remove").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        widgets.remove(i);
    }
    if let Some((a, b)) = swap {
        widgets.swap(a, b);
    }
    ui.horizontal_wrapped(|ui| {
        for (label, widget) in [
            ("+ CPU %", Widget::CpuPercent),
            ("+ CPU bar", Widget::CpuBar),
            ("+ Temp", Widget::Temperature),
            ("+ Disk", Widget::DiskFree),
            (
                "+ Text",
                Widget::Text {
                    value: String::new(),
                },
            ),
        ] {
            if ui.button(label).clicked() {
                widgets.push(widget);
            }
        }
    });
}

const fn widget_name(widget: &Widget) -> &'static str {
    match widget {
        Widget::CpuPercent => "CPU %",
        Widget::CpuBar => "CPU bar",
        Widget::Temperature => "Temperature",
        Widget::DiskFree => "Disk free",
        Widget::Text { .. } => "Text",
    }
}

/// Size of the side strip for `avail_width`: at most the device width, 480:100 ratio.
fn side_strip_size(avail_width: f32) -> Vec2 {
    let width = (avail_width - GRID_MARGIN).clamp(1.0, SIDE_STRIP_MAX);
    Vec2::new(width, width * SIDE_HEIGHT as f32 / SIDE_WIDTH as f32)
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
            let problems = self.session.profile.problems();
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
                let side = self.side_strip(ui, &ctx);
                (side, self.grid(ui, &ctx))
            })
            .inner;
        if let Some(op) = page_op {
            self.apply_page_op(op);
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

/// One edit to the prompt list. The index is the row the user pressed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptOp {
    Up(usize),
    Down(usize),
    Delete(usize),
    Add,
}

/// Apply `op` to the prompt list. Out-of-range rows do nothing.
fn apply_prompt_op(prompts: &mut Vec<Prompt>, op: PromptOp) {
    match op {
        PromptOp::Up(index) if index > 0 && index < prompts.len() => prompts.swap(index - 1, index),
        PromptOp::Down(index) if index + 1 < prompts.len() => prompts.swap(index, index + 1),
        PromptOp::Delete(index) if index < prompts.len() => {
            prompts.remove(index);
        }
        // Empty text fails `problems()`, so the save gate stops until the user fills it in.
        PromptOp::Add => prompts.push(Prompt {
            label: "New".into(),
            text: String::new(),
        }),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_prompt_op, PromptOp};
    use vibe_keyboard::domain::profile::Prompt;

    #[test]
    fn prompt_ops_reorder_delete_and_add() {
        let prompt = |label: &str| Prompt {
            label: label.into(),
            text: format!("/{label}"),
        };
        let mut prompts = vec![prompt("a"), prompt("b"), prompt("c")];
        apply_prompt_op(&mut prompts, PromptOp::Down(0));
        assert_eq!(
            prompts.iter().map(|p| p.label.as_str()).collect::<Vec<_>>(),
            ["b", "a", "c"]
        );
        apply_prompt_op(&mut prompts, PromptOp::Up(0));
        assert_eq!(prompts[0].label, "b", "Up on the first row does nothing");
        apply_prompt_op(&mut prompts, PromptOp::Delete(1));
        assert_eq!(
            prompts.iter().map(|p| p.label.as_str()).collect::<Vec<_>>(),
            ["b", "c"]
        );
        apply_prompt_op(&mut prompts, PromptOp::Delete(9));
        apply_prompt_op(&mut prompts, PromptOp::Add);
        assert_eq!(prompts.len(), 3);
        assert_eq!(prompts[2].text, "");
    }

    use super::{
        cell_size, parse_lsappinfo, side_strip_size, Session, GAP, GRID_MARGIN, KEY_PANEL_WIDTH,
        PAGES_WIDTH, UNDO_WINDOW,
    };
    use eframe::egui::Vec2;
    use std::time::{Duration, Instant};
    use vibe_keyboard::domain::profile::{KeyConfig, Profile};

    /// A layout with one extra page, so it differs from the default.
    fn edited_layout() -> Profile {
        let mut profile = Profile::default_from_legacy();
        profile.add_page("Extra");
        profile
    }

    #[test]
    fn discard_is_refused_when_clean() {
        let mut session = Session::new(Profile::default_from_legacy());
        let now = Instant::now();
        assert!(!session.dirty());
        assert!(!session.discard(Profile::default_from_legacy(), now));
        assert!(!session.undo_available(now));
    }

    #[test]
    fn dirty_then_discard_then_undo_returns_to_dirty() {
        let disk = Profile::default_from_legacy();
        let mut session = Session::new(disk.clone());
        session.profile = edited_layout();
        let now = Instant::now();
        assert!(session.dirty());

        assert!(session.discard(disk.clone(), now));
        assert!(!session.dirty());
        assert_eq!(session.profile, disk);
        assert!(session.undo_available(now));

        assert!(session.undo(now));
        assert!(session.dirty());
        assert_eq!(session.profile, edited_layout());
        assert!(!session.undo_available(now));
    }

    #[test]
    fn undo_expires_after_the_window() {
        let disk = Profile::default_from_legacy();
        let mut session = Session::new(disk.clone());
        session.profile = edited_layout();
        let start = Instant::now();
        assert!(session.discard(disk.clone(), start));
        let later = start + UNDO_WINDOW + Duration::from_millis(1);
        assert!(!session.undo_available(later));
        session.expire(later);
        assert!(!session.undo(later));
        assert_eq!(session.profile, disk);
    }

    #[test]
    fn save_clears_the_undo() {
        let disk = Profile::default_from_legacy();
        let mut session = Session::new(disk.clone());
        session.profile = edited_layout();
        let now = Instant::now();
        assert!(session.discard(disk, now));
        session.mark_saved();
        assert!(!session.undo_available(now));
    }

    #[test]
    fn a_single_key_edit_makes_the_session_dirty() {
        let mut session = Session::new(Profile::default_from_legacy());
        session.profile.set_key(
            0,
            7,
            KeyConfig {
                label: "x".into(),
                ..KeyConfig::default()
            },
        );
        assert!(session.dirty());
    }

    #[test]
    fn grid_fits_the_default_and_minimum_window() {
        // Window minus both side panels, the top and bottom bars, and the side strip.
        let central = |window: Vec2| {
            let width = window.x - PAGES_WIDTH - KEY_PANEL_WIDTH;
            let strip = side_strip_size(width).y + GAP;
            Vec2::new(width, window.y - 60.0 - strip)
        };
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
