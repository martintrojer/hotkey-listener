//! Cross-platform global hotkey listener with native Wayland support.
//!
//! This crate provides a simple API for listening to global hotkeys on Linux and macOS.
//! Unlike other crates, it uses evdev directly on Linux, making it compatible with
//! both X11 and Wayland.
//!
//! # Features
//!
//! - **Native Wayland support on Linux** - Uses evdev directly (reads `/dev/input`)
//! - **Automatic keyboard reconnection** - Handles USB keyboard disconnect/reconnect
//! - **Modifier key support** - Parse and detect `Shift+F8` style hotkey combinations
//! - **Simple push-to-talk API** - Clean pressed/released event model
//! - **Cross-platform** - Linux (evdev) + macOS (rdev) with unified API
//!
//! # Example
//!
//! ```no_run
//! use hotkey_listener::{parse_hotkey, HotkeyListenerBuilder};
//! use std::sync::Arc;
//! use std::sync::atomic::AtomicBool;
//!
//! fn main() -> anyhow::Result<()> {
//!     let hotkey = parse_hotkey("Shift+F8")?;
//!
//!     let running = Arc::new(AtomicBool::new(true));
//!     let listener = HotkeyListenerBuilder::new()
//!         .add_hotkey(hotkey)
//!         .build()?;
//!
//!     let rx = listener.start(running.clone())?;
//!
//!     while let Ok(event) = rx.recv() {
//!         println!("Event: {:?}", event);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Linux Requirements
//!
//! On Linux, the user must have permission to read from `/dev/input/event*` devices.
//! This typically means running as root or being a member of the `input` group.

mod event;
mod hotkey;
mod key;
mod listener;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

pub use event::HotkeyEvent;
pub use hotkey::{parse_hotkey, Hotkey, Modifiers};
pub use key::Key;
pub use listener::{HotkeyListener, HotkeyListenerBuilder};

#[cfg(target_os = "linux")]
pub use linux::find_keyboards;
