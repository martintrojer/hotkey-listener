//! macOS implementation using rdev.

use crate::event::HotkeyEvent;
use crate::hotkey::{Hotkey, Modifiers};
use crate::key::Key;
use anyhow::Result;
use rdev::{listen, Event, EventType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

/// Convert our platform-agnostic Key to rdev Key.
fn to_rdev_key(key: Key) -> rdev::Key {
    match key {
        Key::F1 => rdev::Key::F1,
        Key::F2 => rdev::Key::F2,
        Key::F3 => rdev::Key::F3,
        Key::F4 => rdev::Key::F4,
        Key::F5 => rdev::Key::F5,
        Key::F6 => rdev::Key::F6,
        Key::F7 => rdev::Key::F7,
        Key::F8 => rdev::Key::F8,
        Key::F9 => rdev::Key::F9,
        Key::F10 => rdev::Key::F10,
        Key::F11 => rdev::Key::F11,
        Key::F12 => rdev::Key::F12,
        Key::ScrollLock => rdev::Key::ScrollLock,
        Key::Pause => rdev::Key::Pause,
        Key::Insert => rdev::Key::Insert,
    }
}

/// macOS hotkey listener using rdev.
pub struct HotkeyListener {
    hotkeys: Vec<Hotkey>,
}

impl HotkeyListener {
    /// Create a new listener with the given hotkeys.
    pub fn new(hotkeys: Vec<Hotkey>) -> Self {
        Self { hotkeys }
    }

    /// Start listening for hotkeys in a background thread.
    /// Returns a receiver for hotkey events.
    pub fn start(self, running: Arc<AtomicBool>) -> Result<Receiver<HotkeyEvent>> {
        let (tx, rx) = mpsc::channel();
        start_keyboard_listener(self.hotkeys, running, tx);
        Ok(rx)
    }
}

fn start_keyboard_listener(
    hotkeys: Vec<Hotkey>,
    running: Arc<AtomicBool>,
    tx: Sender<HotkeyEvent>,
) {
    // Convert hotkeys to rdev keys
    let rdev_hotkeys: Vec<(rdev::Key, Modifiers)> = hotkeys
        .iter()
        .map(|h| (to_rdev_key(h.key), h.modifiers))
        .collect();

    thread::spawn(move || {
        let mut current_mods = Modifiers::default();

        let callback = move |event: Event| {
            match event.event_type {
                // Track modifier state
                EventType::KeyPress(key) => {
                    match key {
                        rdev::Key::ShiftLeft | rdev::Key::ShiftRight => {
                            current_mods.shift = true;
                        }
                        rdev::Key::ControlLeft | rdev::Key::ControlRight => {
                            current_mods.ctrl = true;
                        }
                        rdev::Key::Alt => {
                            current_mods.alt = true;
                        }
                        _ => {}
                    }

                    // Check each hotkey
                    for (idx, (hotkey_key, hotkey_mods)) in rdev_hotkeys.iter().enumerate() {
                        if key == *hotkey_key {
                            let mods_match = current_mods.shift == hotkey_mods.shift
                                && current_mods.ctrl == hotkey_mods.ctrl
                                && current_mods.alt == hotkey_mods.alt;

                            if mods_match {
                                let _ = tx.send(HotkeyEvent::Pressed(idx));
                            }
                        }
                    }
                }
                EventType::KeyRelease(key) => {
                    match key {
                        rdev::Key::ShiftLeft | rdev::Key::ShiftRight => {
                            current_mods.shift = false;
                        }
                        rdev::Key::ControlLeft | rdev::Key::ControlRight => {
                            current_mods.ctrl = false;
                        }
                        rdev::Key::Alt => {
                            current_mods.alt = false;
                        }
                        _ => {}
                    }

                    // Check each hotkey for release
                    for (idx, (hotkey_key, hotkey_mods)) in rdev_hotkeys.iter().enumerate() {
                        if key == *hotkey_key {
                            // For release, we don't check modifiers since they might
                            // have been released before the key
                            let _ = tx.send(HotkeyEvent::Released(idx));
                            let _ = hotkey_mods; // suppress unused warning
                        }
                    }
                }
                _ => {}
            }
        };

        if let Err(e) = listen(callback) {
            log::error!("Error listening to keyboard events: {:?}", e);
            running.store(false, Ordering::SeqCst);
        }
    });
}
