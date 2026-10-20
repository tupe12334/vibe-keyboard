//! State that lives across device-loop passes.
use super::run::EventSink;
use super::side::SideMetrics;
use crate::application::Refresh;
use crate::domain::navigation::NavigationStack;
use crate::infrastructure::frontmost;
use crate::infrastructure::images::app_icon_generation;
use crate::infrastructure::persistence::DeviceState;
use crate::presentation::tui::AppState;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

const FRONTMOST_POLL: Duration = Duration::from_millis(500);

pub enum Flow {
    Continue,
    /// Shutdown was requested.
    Stop,
    /// The device stopped answering. The caller reconnects.
    Lost,
}

pub struct Ctx {
    pub app_state: Arc<Mutex<AppState>>,
    pub dev_state: Arc<Mutex<DeviceState>>,
    pub nav: NavigationStack,
    pub shutdown: Arc<AtomicBool>,
    pub events: EventSink,
    pub frontmost_rx: Receiver<String>,
    pub icon_generation: u64,
    pub was_on_main_page: bool,
    /// Main page last seen, to end the holds when the page changes.
    pub last_page: Option<usize>,
    pub last_heartbeat: Instant,
    pub pages: Refresh,
    pub last_profile_mtime: Option<SystemTime>,
    pub last_profile_check: Instant,
    pub side: SideMetrics,
}

impl Ctx {
    /// `last_profile_mtime` must be taken before the device wait, so an edit made while
    /// the deck is unplugged reloads.
    pub fn new(
        app_state: Arc<Mutex<AppState>>,
        dev_state: Arc<Mutex<DeviceState>>,
        nav: NavigationStack,
        shutdown: Arc<AtomicBool>,
        events: EventSink,
        last_profile_mtime: Option<SystemTime>,
    ) -> Self {
        Self {
            was_on_main_page: nav.main_page().is_some(),
            last_page: nav.main_page(),
            nav,
            app_state,
            dev_state,
            shutdown,
            events,
            frontmost_rx: frontmost::watch(FRONTMOST_POLL),
            icon_generation: app_icon_generation(),
            last_heartbeat: Instant::now(),
            pages: Refresh::default(),
            last_profile_mtime,
            last_profile_check: Instant::now(),
            side: SideMetrics::start(),
        }
    }
}
