/// Something the controller loop reports to its caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerEvent {
    DeviceReady,
    DeviceLost,
    DeviceRestored,
    /// The loop has exited. Sent on every exit path.
    Stopped,
    /// A key went down. Logical key number, 1–15.
    KeyPressed(u8),
    /// A new profile file was read and is on the deck.
    ProfileReloaded,
}
