use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui;

use vibe_keyboard::infrastructure::system_metrics::Shared;
use vibe_keyboard::infrastructure::{profile_store, system_metrics};

use super::previews::{Preview, SidePreview};
use super::session::{load_from_disk, Session};

pub(super) struct Editor {
    pub(super) root: PathBuf,
    pub(super) session: Session,
    /// The unsaved-changes dialog is open (window close was requested while dirty).
    pub(super) close_prompt: bool,
    /// File modification time when it was last loaded or written by this editor.
    pub(super) loaded_mtime: Option<std::time::SystemTime>,
    /// The file changed on disk since it was loaded. Save waits for the user's choice.
    pub(super) disk_conflict: bool,
    pub(super) page: usize,
    pub(super) key: Option<u8>,
    pub(super) status: String,
    /// Last rendered preview per key, with the config it was rendered from.
    pub(super) previews: HashMap<u8, Preview>,
    /// Running apps as (name, bundle id), refreshed on demand.
    pub(super) running: Vec<(String, String)>,
    /// Pending result of a background `lsappinfo` run.
    pub(super) running_rx: Option<mpsc::Receiver<Vec<(String, String)>>>,
    pub(super) new_app: String,
    /// Bundle id typed into the app image picker.
    pub(super) image_app_text: String,
    /// Textures of the icon pack, loaded on first use.
    pub(super) icon_textures: HashMap<&'static str, egui::TextureHandle>,
    /// Set when `profile.toml` could not be read. Saving is refused while it is set.
    pub(super) load_error: Option<String>,
    /// Latest system sample, shared with the sampler thread. Widgets in the preview use it.
    pub(super) metrics: Shared,
    /// Last rendered side preview of the current page.
    pub(super) side_preview: Option<SidePreview>,
    /// The side panel is shown instead of the key panel.
    pub(super) side_open: bool,
}

impl Editor {
    pub(super) fn new() -> Self {
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

    pub(super) fn dirty(&self) -> bool {
        self.session.profile != self.session.saved
    }

    /// Keep the selected page inside the current layout. The key selection is dropped when
    /// the page moves.
    pub(super) fn clamp_selection(&mut self) {
        let count = self.session.profile.pages.len();
        let page = self.page.min(count.saturating_sub(1));
        if page != self.page {
            self.page = page;
            self.key = None;
        }
    }
}
