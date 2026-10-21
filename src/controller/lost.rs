//! The device-lost path: end the holds, reconnect, and report each step.
use super::ctx::Ctx;
use super::event::ControllerEvent;
use super::setup::reconnect;
use super::stop::stop_if_none;
use crate::application::release_all_holds;
use crate::domain::ptt::Release;
use crate::infrastructure::usb::{DeckPort, Device};
use std::time::Instant;
use tracing::warn;

/// The deck stopped answering. Returns the new handle, or None when shutdown comes first.
pub fn recover(port: &dyn DeckPort, handle: Device, ctx: &mut Ctx) -> Option<Device> {
    warn!("device disconnected");
    release_all_holds(&ctx.app_state, Release::Disconnected);
    (ctx.events)(ControllerEvent::DeviceLost);
    let new_handle = stop_if_none(reconnect(port, handle, ctx), &*ctx.events)?;
    (ctx.events)(ControllerEvent::DeviceRestored);
    ctx.last_heartbeat = Instant::now();
    Some(new_handle)
}
