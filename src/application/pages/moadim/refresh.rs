//! Timed refresh of the open routines pages, and the results of routine presses.

use std::sync::mpsc::{channel, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::MoadimRefresh;
use crate::application::render_screen;
use crate::domain::moadim::{Flash, MoadimList};
use crate::domain::navigation::NavigationStack;
use crate::infrastructure::moadim::list_routines;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// How often the open routines pages refresh. The request runs on its own thread.
const REFRESH: Duration = Duration::from_secs(10);

impl MoadimRefresh {
    pub fn tick(
        &mut self,
        nav: &mut NavigationStack,
        handle: &Device,
        state: &Arc<Mutex<tui::AppState>>,
        dev_state: &Arc<Mutex<DeviceState>>,
    ) {
        while let Ok(done) = self.done_rx.try_recv() {
            nav.set_flash(Flash {
                key: done.key,
                label: done.label,
                until: Instant::now() + super::actions::MOADIM_FLASH,
            });
            render_screen(nav, handle, state, dev_state);
        }
        if let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok(fresh) => {
                    self.rx = None;
                    let list = fresh.map_or(MoadimList::Offline, MoadimList::Online);
                    if nav.set_moadim(list) {
                        render_screen(nav, handle, state, dev_state);
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => self.rx = None,
            }
        }
        if nav.clear_expired_flash(Instant::now()) {
            render_screen(nav, handle, state, dev_state);
        }
        if !nav.is_moadim() {
            self.rx = None;
            self.started = None;
            return;
        }
        if self.rx.is_none() && self.started.is_none_or(|t| t.elapsed() >= REFRESH) {
            let (tx, rx) = channel();
            std::thread::spawn(move || {
                let _ = tx.send(list_routines());
            });
            self.rx = Some(rx);
            self.started = Some(Instant::now());
        }
    }
}
