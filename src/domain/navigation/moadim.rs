use std::time::Instant;

use crate::domain::moadim::{Flash, MoadimList};

use super::{moadim_len, Screen};

impl super::NavigationStack {
    /// Update the open Moadim screen with a fresh list. Returns true when the screen changed.
    pub fn set_moadim(&mut self, new: MoadimList) -> bool {
        match self.stack.last_mut() {
            Some(Screen::MoadimList { list, page }) => {
                if *list == new {
                    return false;
                }
                let max = moadim_len(&new).saturating_sub(1) / 10;
                *page = (*page).min(max);
                *list = new;
                true
            }
            Some(Screen::MoadimRoutine { routine, .. }) => {
                let MoadimList::Online(routines) = &new else {
                    return false;
                };
                let Some(fresh) = routines.iter().find(|r| r.id == routine.id) else {
                    return false;
                };
                if fresh == routine {
                    return false;
                }
                *routine = fresh.clone();
                true
            }
            _ => false,
        }
    }

    /// True when the top screen shows Moadim routines.
    pub fn is_moadim(&self) -> bool {
        matches!(
            self.stack.last(),
            Some(Screen::MoadimList { .. } | Screen::MoadimRoutine { .. })
        )
    }

    /// Show `flash` on the open routine screen, replacing any flash already there.
    pub fn set_flash(&mut self, flash: Flash) {
        if let Some(Screen::MoadimRoutine { flash: slot, .. }) = self.stack.last_mut() {
            *slot = Some(flash);
        }
    }

    /// Drop the flash on the open routine screen once its time is up. True when it dropped one.
    pub fn clear_expired_flash(&mut self, now: Instant) -> bool {
        match self.stack.last_mut() {
            Some(Screen::MoadimRoutine { flash, .. })
                if flash.as_ref().is_some_and(|f| now >= f.until) =>
            {
                *flash = None;
                true
            }
            _ => false,
        }
    }
}
