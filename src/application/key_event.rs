use std::sync::{Arc, Mutex};

use tracing::info;

use super::gestures::end_press;
use super::holds::track_press;
use super::{handle_action_key, lock, moadim_keys, prompt_return, render_screen, Refresh};
use crate::domain::keys::raw_to_logical;
use crate::domain::navigation::{back_key_on, BackKey, NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

pub fn handle_key_event(
    buf: &[u8],
    handle: &Device,
    nav: &mut NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
    refresh: &mut Refresh,
) {
    let raw_id = buf[9];
    let state_byte = buf[10];
    if raw_id == 0x00 || raw_id == 0xFF {
        return;
    }
    let state_str = match state_byte {
        1 => "pressed",
        0 => "released",
        _ => return,
    };
    let Some(key) = raw_to_logical(raw_id) else {
        return;
    };
    // Release always clears the highlight, even while loading.
    if state_byte != 1 {
        lock(state).pressed_key = None;
        end_press(key, handle, nav, state, dev_state);
        return;
    }
    if lock(state).loading {
        return;
    }
    lock(state).pressed_key = Some(key);
    info!("key {:2} {state_str}", key);
    track_press(key, nav, state);

    let is_input_number = matches!(nav.current(), Screen::InputNumber { .. });
    if matches!(key, 11 | 13) && nav.main_page().is_some() {
        // Paging by hand on a main page makes it manual, so app switches stop fighting it.
        lock(state).auto_page.on_manual_page();
    }

    match key {
        11 => {
            match back_key_on(nav.current()) {
                BackKey::Back => nav.back(),
                BackKey::Out => nav.out_to_main(),
            }
            render_screen(nav, handle, state, dev_state);
        }
        12 => {
            // On an action page, key 12 goes back to the session list. Elsewhere it goes out.
            if matches!(nav.current(), Screen::SessionActions { .. }) {
                prompt_return::back_to_sessions(nav);
            } else {
                nav.out();
            }
            render_screen(nav, handle, state, dev_state);
        }
        // On the InputNumber screen key 13 maps to digit 7, not "forward".
        _ if nav.is_moadim() => {
            moadim_keys::handle_key(key, nav, handle, state, dev_state, &mut refresh.moadim);
        }
        13 if !is_input_number
            && !matches!(
                nav.current(),
                Screen::ClaudeSessions { .. }
                    | Screen::SessionActions { .. }
                    | Screen::ConfirmExit { .. }
            ) =>
        {
            nav.forward();
            render_screen(nav, handle, state, dev_state);
        }
        _ => handle_action_key(key, nav, handle, state, dev_state),
    }
}
