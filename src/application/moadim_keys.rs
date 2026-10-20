//! Key presses on the Moadim routines pages: the routine list and one routine's actions.

use std::sync::{Arc, Mutex};

use tracing::info;

use crate::application::{render_screen, MoadimRefresh};
use crate::domain::moadim::{MoadimList, Routine};
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::moadim::RoutineAction;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// What a key press asks for. Read from the screen before the stack changes.
enum Pick {
    Open(Routine),
    Press {
        id: String,
        verb: RoutineAction,
        label: &'static str,
    },
    Page {
        forward: bool,
    },
    Nothing,
}

/// Handles key `key` on a Moadim page. Keys 11 and 12 never reach here.
pub fn handle_key(
    key: u8,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
    moadim: &mut MoadimRefresh,
) {
    let pick = match nav.current() {
        Screen::MoadimList { list, page } => match (key, list) {
            (1..=10, MoadimList::Online(routines)) => routines
                .get(page * 10 + (key as usize - 1))
                .map_or(Pick::Nothing, |routine| Pick::Open(routine.clone())),
            (14, _) => Pick::Page { forward: false },
            (15, _) => Pick::Page { forward: true },
            _ => Pick::Nothing,
        },
        Screen::MoadimRoutine { routine, .. } => {
            let (verb, label) = match key {
                1 => (RoutineAction::Trigger, "STARTED"),
                2 if routine.enabled => (RoutineAction::SetEnabled(false), "DISABLED"),
                2 => (RoutineAction::SetEnabled(true), "ENABLED"),
                3 => (RoutineAction::Snooze, "SNOOZED"),
                _ => return,
            };
            Pick::Press {
                id: routine.id.clone(),
                verb,
                label,
            }
        }
        _ => Pick::Nothing,
    };
    match pick {
        Pick::Open(routine) => {
            info!("moadim: routine {}", routine.title);
            nav.push(Screen::MoadimRoutine {
                routine,
                flash: None,
            });
            render_screen(nav, handle, state, dev_state);
        }
        Pick::Press { id, verb, label } => {
            info!("moadim: {verb:?} on {id}");
            // Off the device loop: the daemon call can take up to its request timeout.
            moadim.press(key, id, verb, label);
        }
        Pick::Page { forward } => {
            if forward {
                nav.forward();
            } else {
                nav.previous_page();
            }
            render_screen(nav, handle, state, dev_state);
        }
        Pick::Nothing => {}
    }
}
