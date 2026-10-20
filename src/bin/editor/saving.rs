//! Saving the layout from the editor: the shared rules, then compare-and-write.
use std::time::SystemTime;

use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::domain::profile_rules;
use vibe_keyboard::infrastructure::profile_save::{save_if_unchanged, SaveRefusal};
use vibe_keyboard::infrastructure::profile_store;

/// What a save attempt did.
pub enum Outcome {
    Saved,
    /// The file changed on disk since it was loaded. Nothing was written.
    Conflict,
    /// Nothing was written. The text says why.
    Refused(String),
}

/// Write `profile`. Refuses while the layout has problems. Unless `overwrite`, refuses when the
/// file changed since `loaded`. With `overwrite`, the write goes over the disk version.
pub fn write(profile: &Profile, loaded: Option<SystemTime>, overwrite: bool) -> Outcome {
    let problems = profile_rules::save_problems(profile);
    if !problems.is_empty() {
        return Outcome::Refused(format!(
            "Cannot save: {} problem(s) in the layout. Fix them first (listed below).",
            problems.len()
        ));
    }
    let path = profile_store::profile_path();
    let disk = profile_store::modified_at(&path);
    let base = if overwrite { disk } else { loaded };
    match save_if_unchanged(&path, profile, base) {
        Ok(_) => Outcome::Saved,
        Err(SaveRefusal::Changed) => Outcome::Conflict,
        Err(SaveRefusal::Failed(e)) => Outcome::Refused(format!("Save failed: {e}")),
    }
}
