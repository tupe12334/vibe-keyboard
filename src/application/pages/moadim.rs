//! The Moadim routines list: one tile per routine, paged like the sessions list. The daemon is
//! asked on a worker thread, so the device loop never waits for it.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::application::render_screen;
use crate::domain::moadim::{MoadimList, Routine, TriggerDebounce};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;
use tracing::info;

mod actions;
mod list;
mod press;
mod refresh;

/// A routine action that finished on its worker thread. `label` is the tile text to show.
struct Done {
    key: u8,
    label: &'static str,
}

/// Keeps the open routines pages current, and reports routine presses. Call `tick` once per loop
/// turn. It never blocks.
pub struct MoadimRefresh {
    started: Option<Instant>,
    rx: Option<Receiver<Option<Vec<Routine>>>>,
    done_tx: Sender<Done>,
    done_rx: Receiver<Done>,
    debounce: TriggerDebounce,
}

impl Default for MoadimRefresh {
    fn default() -> Self {
        let (done_tx, done_rx) = channel();
        Self {
            started: None,
            rx: None,
            done_tx,
            done_rx,
            debounce: TriggerDebounce::default(),
        }
    }
}

impl MoadimRefresh {}

/// Draw a Moadim page. Only the two Moadim screens reach here.
pub fn render(screen: &Screen, handle: &Device, state: &Arc<Mutex<tui::AppState>>) {
    match screen {
        Screen::MoadimList { list, page } => list::render_moadim_list(list, *page, handle, state),
        Screen::MoadimRoutine { routine, flash } => {
            actions::render_moadim_routine(routine, flash.as_ref(), handle, state);
        }
        _ => {}
    }
}

/// Opens the routines list. The built-in `moadim` profile entry runs this.
pub fn open_routines(
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    info!("opening Moadim routines");
    nav.push(Screen::MoadimList {
        list: MoadimList::Pending,
        page: 0,
    });
    render_screen(nav, handle, state, dev_state);
}
