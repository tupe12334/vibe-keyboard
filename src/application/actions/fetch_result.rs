//! What a finished page fetch means for the screen: show it, drop it, or keep waiting.

use std::sync::mpsc::{Receiver, TryRecvError};

use tracing::info;

use crate::domain::actions::{CentyProject, ClaudeSession};
use crate::domain::navigation::{Screen, ScreenId};

/// A page's list, fetched on a background thread.
pub(super) enum Fetched {
    Projects(Vec<CentyProject>),
    Sessions(Vec<ClaudeSession>),
}

/// A fetch in flight, and the screen that was open when it started.
pub(super) struct Pending {
    pub(super) rx: Receiver<Fetched>,
    pub(super) from: ScreenId,
}

/// What the device loop does with the fetch in flight.
pub(super) enum Ready {
    /// Nothing is in flight.
    Idle,
    /// Still running.
    Waiting,
    /// Landed, or lost. Show this page, if any.
    Done(Option<Screen>),
}

/// Take the fetch result if it is ready. Clears the slot once it lands or is lost. A result
/// for a screen the user has left is dropped.
pub(super) fn take_ready(slot: &mut Option<Pending>, now: ScreenId) -> Ready {
    let Some(pending) = slot.as_ref() else {
        return Ready::Idle;
    };
    let fetched = match pending.rx.try_recv() {
        Ok(fetched) => Some(fetched),
        Err(TryRecvError::Empty) => return Ready::Waiting,
        // The fetch thread panicked or timed out without sending. Clear loading, show nothing.
        Err(TryRecvError::Disconnected) => None,
    };
    let stale = pending.from != now;
    *slot = None;
    if stale {
        return Ready::Done(None);
    }
    Ready::Done(fetched.and_then(next_screen))
}

/// The page to show for a fetch. None keeps the current page (no projects found).
fn next_screen(fetched: Fetched) -> Option<Screen> {
    match fetched {
        Fetched::Projects(projects) if projects.is_empty() => None,
        Fetched::Projects(projects) => Some(Screen::CentyProjectList {
            projects,
            page: 0,
            filter: None,
        }),
        Fetched::Sessions(sessions) => {
            info!("claude sessions: {} found", sessions.len());
            Some(Screen::ClaudeSessions { sessions, page: 0 })
        }
    }
}

#[cfg(test)]
#[path = "fetch_result_tests.rs"]
mod tests;
