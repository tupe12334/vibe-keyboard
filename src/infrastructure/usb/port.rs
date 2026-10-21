//! What the controller needs from the deck: its saved state, a handle, retry pacing,
//! and the stuck-button clear. `Hardware` is the real one. Tests pass a fake.
use super::reconnect::RETRY_INTERVAL;
use super::Device;
use crate::infrastructure::mouse_release::clear_stuck_left_button;
use crate::infrastructure::persistence::DeviceState;
use std::time::Duration;

pub trait DeckPort {
    fn load_state(&self) -> DeviceState;
    fn open(&self) -> Result<Device, String>;
    fn retry_interval(&self) -> Duration;
    fn clear_stuck_button(&self);
}

/// The real deck and the real OS.
pub struct Hardware;

impl DeckPort for Hardware {
    fn load_state(&self) -> DeviceState {
        DeviceState::load()
    }

    fn open(&self) -> Result<Device, String> {
        Device::open()
    }

    fn retry_interval(&self) -> Duration {
        RETRY_INTERVAL
    }

    fn clear_stuck_button(&self) {
        clear_stuck_left_button();
    }
}
