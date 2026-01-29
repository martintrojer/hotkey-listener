//! Platform-agnostic listener builder.

use crate::event::HotkeyEvent;
use crate::hotkey::Hotkey;
use anyhow::Result;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

/// Builder for creating a hotkey listener.
#[derive(Default)]
pub struct HotkeyListenerBuilder {
    hotkeys: Vec<Hotkey>,
}

impl HotkeyListenerBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a hotkey to listen for.
    pub fn add_hotkey(mut self, hotkey: Hotkey) -> Self {
        self.hotkeys.push(hotkey);
        self
    }

    /// Build the listener.
    #[cfg(target_os = "linux")]
    pub fn build(self) -> Result<HotkeyListener> {
        let keyboards = crate::linux::find_keyboards()?;
        Ok(HotkeyListener {
            inner: crate::linux::HotkeyListener::new(keyboards, self.hotkeys),
        })
    }

    /// Build the listener.
    #[cfg(target_os = "macos")]
    pub fn build(self) -> Result<HotkeyListener> {
        Ok(HotkeyListener {
            inner: crate::macos::HotkeyListener::new(self.hotkeys),
        })
    }

    /// Build the listener (unsupported platform stub).
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn build(self) -> Result<HotkeyListener> {
        anyhow::bail!("Hotkey listening is not supported on this platform")
    }
}

/// A hotkey listener that runs in a background thread.
pub struct HotkeyListener {
    #[cfg(target_os = "linux")]
    inner: crate::linux::HotkeyListener,
    #[cfg(target_os = "macos")]
    inner: crate::macos::HotkeyListener,
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    inner: (),
}

impl HotkeyListener {
    /// Start listening for hotkeys in a background thread.
    /// Returns a receiver for hotkey events.
    ///
    /// The listener will continue running until the `running` flag is set to false.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn start(self, running: Arc<AtomicBool>) -> Result<Receiver<HotkeyEvent>> {
        self.inner.start(running)
    }

    /// Start listening (unsupported platform stub).
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn start(self, _running: Arc<AtomicBool>) -> Result<Receiver<HotkeyEvent>> {
        anyhow::bail!("Hotkey listening is not supported on this platform")
    }
}
