//! Side-screen widgets: sample system metrics and redraw the side screen.
use super::ctx::Ctx;
use crate::application::refresh_side_widgets;
use crate::infrastructure::system_metrics::{self, Shared};
use crate::infrastructure::usb::Device;
use std::time::{Duration, Instant};

/// Side widgets refresh at this rate. Matches the metrics sampler.
const SIDE_POLL: Duration = Duration::from_secs(2);

/// Sampler state. The sampler thread starts once, in `start`.
pub struct SideMetrics {
    shared: Shared,
    last_tick: Instant,
}

impl SideMetrics {
    pub fn start() -> Self {
        Self {
            shared: system_metrics::start(),
            last_tick: Instant::now(),
        }
    }
}

/// Every `SIDE_POLL`: copy the latest sample into the app state and redraw the side screen.
pub fn tick(c: &mut Ctx, handle: &Device) {
    if c.side.last_tick.elapsed() < SIDE_POLL {
        return;
    }
    c.side.last_tick = Instant::now();
    let sample = system_metrics::snapshot(&c.side.shared);
    c.app_state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .metrics = sample;
    refresh_side_widgets(handle, &c.app_state, &c.nav);
}
