use std::time::{Duration, Instant};

use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::infrastructure::profile_store;

/// How long the Undo for a discard stays available.
pub(super) const UNDO_WINDOW: Duration = Duration::from_secs(10);

/// Edits in memory and the last file state. Pure, so its transitions are unit tested.
#[derive(Default)]
pub(super) struct Session {
    pub(super) profile: Profile,
    /// The profile as last read from or written to disk.
    pub(super) saved: Profile,
    /// The edits dropped by the last discard, and when they were dropped.
    pub(super) dropped: Option<(Profile, Instant)>,
}

impl Session {
    pub(super) fn new(profile: Profile) -> Self {
        Self {
            saved: profile.clone(),
            profile,
            dropped: None,
        }
    }

    pub(super) fn dirty(&self) -> bool {
        self.profile != self.saved
    }

    /// A save wrote `profile` to disk. Earlier discards can no longer be undone.
    pub(super) fn mark_saved(&mut self) {
        self.saved = self.profile.clone();
        self.dropped = None;
    }

    /// Drop unsaved edits and adopt `disk`, the file as it is now. False when nothing is dirty.
    /// Replace everything with the file as it is on disk. Used when the user chooses Reload.
    pub(super) fn replace_with_disk(&mut self, disk: Profile) {
        self.saved = disk.clone();
        self.profile = disk;
        self.dropped = None;
    }

    pub(super) fn discard(&mut self, disk: Profile, now: Instant) -> bool {
        if !self.dirty() {
            return false;
        }
        let dropped = std::mem::replace(&mut self.profile, disk.clone());
        self.dropped = Some((dropped, now));
        self.saved = disk;
        true
    }

    pub(super) fn undo_available(&self, now: Instant) -> bool {
        self.dropped
            .as_ref()
            .is_some_and(|(_, at)| now.duration_since(*at) <= UNDO_WINDOW)
    }

    /// Restore the edits dropped by the last discard, if still in the undo window.
    pub(super) fn undo(&mut self, now: Instant) -> bool {
        if !self.undo_available(now) {
            self.dropped = None;
            return false;
        }
        if let Some((profile, _)) = self.dropped.take() {
            self.profile = profile;
        }
        true
    }

    pub(super) fn expire(&mut self, now: Instant) {
        if !self.undo_available(now) {
            self.dropped = None;
        }
    }
}

/// Read the profile the way startup does. Returns the layout and the read error, if any.
pub(super) fn load_from_disk() -> (Profile, Option<String>) {
    match profile_store::ensure_default(&profile_store::profile_path()) {
        Ok(profile) => (profile, None),
        Err(e) => (Profile::default_from_legacy(), Some(e)),
    }
}
