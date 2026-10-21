use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::domain::actions::ClaudeSession;
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::claude_sessions::{age_label, list_sessions};
use crate::infrastructure::focused_session::with_focus;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
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

/// True when the page would look the same: same tty, label, status, age and Remote Control as shown.
fn same_view(old: &[ClaudeSession], new: &[ClaudeSession]) -> bool {
    old.len() == new.len()
        && old.iter().zip(new).all(|(a, b)| {
            a.tty == b.tty
                && a.focused == b.focused
                && a.remote_control == b.remote_control
                && a.label == b.label
                && a.status == b.status
                && age_label(a.age) == age_label(b.age)
        })
}
