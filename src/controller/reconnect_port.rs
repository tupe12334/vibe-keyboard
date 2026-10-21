//! Fake deck port for the reconnect test: hands out scripted handles and counts its calls.
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::{DeckPort, Device};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::time::Duration;

pub struct FakeDeck {
    handles: RefCell<VecDeque<Device>>,
    pub opens: Cell<u32>,
    pub clears: Cell<u32>,
}

impl FakeDeck {
    pub fn new(handles: Vec<Device>) -> Self {
        Self {
            handles: RefCell::new(handles.into()),
            opens: Cell::new(0),
            clears: Cell::new(0),
        }
    }
}

impl DeckPort for FakeDeck {
    fn load_state(&self) -> DeviceState {
        DeviceState::default()
    }

    fn open(&self) -> Result<Device, String> {
        self.opens.set(self.opens.get() + 1);
        self.handles
            .borrow_mut()
            .pop_front()
            .ok_or("no fake device left".to_string())
    }

    fn retry_interval(&self) -> Duration {
        Duration::ZERO
    }

    fn clear_stuck_button(&self) {
        self.clears.set(self.clears.get() + 1);
    }
}
