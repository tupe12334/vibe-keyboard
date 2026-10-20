use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ClaudeSession, ScreenView, SessionStatus};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::claude_sessions::{age_label, list_sessions};
use crate::infrastructure::focused_session::with_focus;
use crate::infrastructure::images::{label_image, session_image};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// How often the open sessions page rescans. The scan runs on its own thread.
const REFRESH: Duration = Duration::from_secs(5);

/// Keeps the open sessions page current. Call `tick` once per loop turn. It never blocks.
#[derive(Default)]
pub struct SessionsRefresh {
    started: Option<Instant>,
    rx: Option<Receiver<Vec<ClaudeSession>>>,
}

impl SessionsRefresh {
    pub fn tick(
        &mut self,
        nav: &mut NavigationStack,
        handle: &Device,
        state: &Arc<Mutex<tui::AppState>>,
        dev_state: &Arc<Mutex<DeviceState>>,
    ) {
        if let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok(fresh) => {
                    self.rx = None;
                    if let Screen::ClaudeSessions { sessions, .. } = nav.current() {
                        if !same_view(sessions, &fresh) {
                            nav.set_sessions(fresh);
                            crate::application::render_screen(nav, handle, state, dev_state);
                        }
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => self.rx = None,
            }
        }
        if !matches!(nav.current(), Screen::ClaudeSessions { .. }) {
            self.rx = None;
            self.started = None;
            return;
        }
        if self.rx.is_none() && self.started.is_none_or(|t| t.elapsed() >= REFRESH) {
            let (tx, rx) = channel();
            std::thread::spawn(move || tx.send(with_focus(list_sessions())).ok());
            self.rx = Some(rx);
            self.started = Some(Instant::now());
        }
    }
}

/// True when the page would look the same: same tty, label, status and age as shown.
fn same_view(old: &[ClaudeSession], new: &[ClaudeSession]) -> bool {
    old.len() == new.len()
        && old.iter().zip(new).all(|(a, b)| {
            a.tty == b.tty
                && a.focused == b.focused
                && a.label == b.label
                && a.status == b.status
                && age_label(a.age) == age_label(b.age)
        })
}

/// Keys 1..=10 show sessions, one tile each. Key 14 is BACK after the first page. Key 15 shows
/// `MORE` or `PAGE` with the page counter when there is more than one page.
pub fn render_claude_sessions(
    sessions: &[ClaudeSession],
    page: usize,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    const PER_PAGE: usize = 10;
    let pages = sessions.len().div_ceil(PER_PAGE).max(1);
    let inset = super::key_tiles::key_inset(state);
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    let mut tiles = Vec::new();
    for (i, entry) in sessions
        .iter()
        .skip(page * PER_PAGE)
        .take(PER_PAGE)
        .enumerate()
    {
        let key = (i + 1) as u8;
        actions.insert(
            key,
            ButtonAction {
                name: entry.label.clone(),
                title: status_text(entry.status).into(),
                description: format!("/dev/{}", entry.tty),
            },
        );
        let age = age_label(entry.age);
        let tile = session_image(&entry.label, entry.status, &age, &inset, entry.focused);
        tiles.push((key, tile));
    }
    if page > 0 {
        actions.insert(
            14,
            ButtonAction {
                name: "Back".into(),
                title: "Previous Sessions".into(),
                description: "Previous page of sessions".into(),
            },
        );
        tiles.push((14, label_image("BACK", &inset)));
    }
    actions.insert(
        13,
        ButtonAction {
            name: "New".into(),
            title: "New Session".into(),
            description: "Open a new Claude Code window".into(),
        },
    );
    tiles.push((13, label_image("NEW +", &inset)));
    if pages > 1 {
        actions.insert(
            15,
            ButtonAction {
                name: "More".into(),
                title: "Next Sessions".into(),
                description: "Next page of sessions".into(),
            },
        );
        let word = if page + 1 < pages { "MORE" } else { "PAGE" };
        let label = format!("{word} {}/{pages}", page + 1);
        tiles.push((15, label_image(&label, &inset)));
    }

    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.actions = actions;
        s.screen = ScreenView::ClaudeSessions {
            total: sessions.len(),
            page,
        };
    }

    for (key, img) in tiles {
        send_button_image(handle, key, &DynamicImage::ImageRgb8(img));
    }
}

/// Short status for the button description, shown in the editor and the TUI.
const fn status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "Active",
        SessionStatus::Stale => "Stale",
    }
}
