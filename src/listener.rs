//! Platform-agnostic listener builder.

use crate::event::HotkeyEvent;
use crate::hotkey::Hotkey;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvError, RecvTimeoutError};
use std::sync::Arc;
use std::time::Duration;

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
    ///
    /// Returns a [`HotkeyListenerHandle`] that receives hotkey events.
    /// The background thread automatically stops when the handle is dropped.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn start(self) -> Result<HotkeyListenerHandle> {
        let running = Arc::new(AtomicBool::new(true));
        let rx = self.inner.start(Arc::clone(&running))?;
        Ok(HotkeyListenerHandle { running, rx })
    }

    /// Start listening (unsupported platform stub).
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn start(self) -> Result<HotkeyListenerHandle> {
        anyhow::bail!("Hotkey listening is not supported on this platform")
    }
}

/// Handle for receiving hotkey events.
///
/// The background listener thread automatically stops when this handle is dropped,
/// providing automatic cleanup without requiring manual shutdown signals.
///
/// # Example
///
/// ```no_run
/// use hotkey_listener::{parse_hotkey, HotkeyListenerBuilder};
/// use std::time::Duration;
///
/// let hotkey = parse_hotkey("F8").unwrap();
/// let handle = HotkeyListenerBuilder::new()
///     .add_hotkey(hotkey)
///     .build()
///     .unwrap()
///     .start()
///     .unwrap();
///
/// // Receive events
/// while let Ok(event) = handle.recv_timeout(Duration::from_millis(100)) {
///     println!("Event: {:?}", event);
/// }
///
/// // Thread stops automatically when handle goes out of scope
/// ```
pub struct HotkeyListenerHandle {
    running: Arc<AtomicBool>,
    rx: Receiver<HotkeyEvent>,
}

impl HotkeyListenerHandle {
    /// Block until the next hotkey event.
    pub fn recv(&self) -> Result<HotkeyEvent, RecvError> {
        self.rx.recv()
    }

    /// Wait for the next hotkey event with a timeout.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<HotkeyEvent, RecvTimeoutError> {
        self.rx.recv_timeout(timeout)
    }

    /// Try to receive a hotkey event without blocking.
    pub fn try_recv(&self) -> Result<HotkeyEvent, std::sync::mpsc::TryRecvError> {
        self.rx.try_recv()
    }

    /// Check if the listener is still running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Manually stop the listener.
    ///
    /// This is called automatically when the handle is dropped.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for HotkeyListenerHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
