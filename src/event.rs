//! Events emitted by the hotkey listener.

/// Events emitted when a registered hotkey is pressed or released.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    /// A hotkey was pressed. The index corresponds to the order in which
    /// hotkeys were added to the listener builder.
    Pressed(usize),
    /// A hotkey was released. The index corresponds to the order in which
    /// hotkeys were added to the listener builder.
    Released(usize),
}
