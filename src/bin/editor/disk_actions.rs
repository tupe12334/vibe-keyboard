use std::time::Instant;

use vibe_keyboard::infrastructure::profile_store;

use super::{load_from_disk, saving};

impl super::Editor {
    /// Write the layout to disk. Returns true when the file was written.
    pub(super) fn save(&mut self) -> bool {
        self.write_file(false)
    }

    /// Write the layout. Refuses on load errors, layout problems, or (unless `overwrite`)
    /// when the file changed on disk since it was loaded.
    pub(super) fn write_file(&mut self, overwrite: bool) -> bool {
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
    pub(super) fn adopt_disk(&mut self) {
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
    pub(super) fn discard(&mut self) {
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

    pub(super) fn undo(&mut self) {
        if self.session.undo(Instant::now()) {
            self.previews.clear();
            self.clamp_selection();
            self.side_preview = None;
            self.status = "Discard undone".into();
        }
    }
}
