use std::sync::{Arc, Mutex};

use image::{DynamicImage, RgbImage};

use crate::domain::navigation::NavigationStack;
use crate::domain::profile::{SideConfig, SideInset};
use crate::domain::side::{resolve_side, SideContent};
use crate::infrastructure::images::{side_image, widget_texts};
use crate::infrastructure::profile_store;
use crate::infrastructure::system_metrics::Metrics;
use crate::infrastructure::usb::{send_side_image, Device};
use crate::presentation::tui;

/// The side screen `nav` shows, with the metrics it reads. None keeps the side screen.
/// The home page is the main page the stack opened from.
fn side_to_show(
    state: &Arc<Mutex<tui::AppState>>,
    nav: &NavigationStack,
) -> Option<(SideConfig, Metrics, SideInset)> {
    let s = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let page_side = s
        .profile
        .pages
        .get(nav.home_page())
        .and_then(|p| p.side.clone());
    match resolve_side(page_side, nav.current().side()) {
        SideContent::Show(side) => Some((side, s.metrics.clone(), s.profile.display.side_inset)),
        SideContent::Keep => None,
    }
}

/// Sends `frame` unless the device already shows it. Returns whether the device shows it now.
/// `side_frame` is set only after the send succeeds, so a failed write is retried. `side_frame`
/// is cleared on every full redraw, because that redraw blanks the side screen too.
fn send_side_frame(handle: &Device, state: &Arc<Mutex<tui::AppState>>, frame: &RgbImage) -> bool {
    {
        let s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if s.side_frame.as_deref() == Some(frame.as_raw().as_slice()) {
            return true;
        }
    }
    if !send_side_image(handle, DynamicImage::ImageRgb8(frame.clone())) {
        return false;
    }
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .side_frame = Some(frame.as_raw().clone());
    true
}

/// Draws the side screen `nav` shows. Called after every full redraw.
pub fn render_side(handle: &Device, state: &Arc<Mutex<tui::AppState>>, nav: &NavigationStack) {
    let Some((side, metrics, inset)) = side_to_show(state, nav) else {
        return;
    };
    let frame = side_image(Some(&side), &profile_store::root_dir(), &metrics, &inset);
    send_side_frame(handle, state, &frame);
}

/// Re-sends the side screen when it has widgets and their frame changed. Called on the
/// controller tick, on main pages and on built-in screens.
pub fn refresh_side_widgets(
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    nav: &NavigationStack,
) {
    let Some((side, metrics, inset)) =
        side_to_show(state, nav).filter(|(side, _, _)| !side.widgets.is_empty())
    else {
        return;
    };
    let texts = widget_texts(&side.widgets, &metrics);
    if state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .side_texts
        .as_deref()
        == Some(texts.as_slice())
    {
        return;
    }
    let frame = side_image(Some(&side), &profile_store::root_dir(), &metrics, &inset);
    if send_side_frame(handle, state, &frame) {
        state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .side_texts = Some(texts);
    }
}
