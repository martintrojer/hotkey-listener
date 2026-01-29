//! Linux implementation using evdev.

use crate::event::HotkeyEvent;
use crate::hotkey::{Hotkey, Modifiers};
use crate::key::Key;
use anyhow::{anyhow, Context, Result};
use evdev::Device;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Convert our platform-agnostic Key to evdev Key.
fn to_evdev_key(key: Key) -> evdev::Key {
    match key {
        Key::F1 => evdev::Key::KEY_F1,
        Key::F2 => evdev::Key::KEY_F2,
        Key::F3 => evdev::Key::KEY_F3,
        Key::F4 => evdev::Key::KEY_F4,
        Key::F5 => evdev::Key::KEY_F5,
        Key::F6 => evdev::Key::KEY_F6,
        Key::F7 => evdev::Key::KEY_F7,
        Key::F8 => evdev::Key::KEY_F8,
        Key::F9 => evdev::Key::KEY_F9,
        Key::F10 => evdev::Key::KEY_F10,
        Key::F11 => evdev::Key::KEY_F11,
        Key::F12 => evdev::Key::KEY_F12,
        Key::ScrollLock => evdev::Key::KEY_SCROLLLOCK,
        Key::Pause => evdev::Key::KEY_PAUSE,
        Key::Insert => evdev::Key::KEY_INSERT,
    }
}

/// Find all keyboard devices in /dev/input.
pub fn find_keyboards() -> Result<Vec<Device>> {
    let mut keyboards = Vec::new();

    for entry in std::fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();

        if !path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("event"))
            .unwrap_or(false)
        {
            continue;
        }

        if let Ok(device) = Device::open(&path) {
            // Check if device supports keyboard keys
            if device
                .supported_keys()
                .map(|keys| keys.contains(evdev::Key::KEY_A))
                .unwrap_or(false)
            {
                log::debug!("Found keyboard: {:?} at {:?}", device.name(), path);
                keyboards.push(device);
            }
        }
    }

    if keyboards.is_empty() {
        Err(anyhow!(
            "No keyboards found. Make sure you're in the 'input' group or running as root."
        ))
    } else {
        Ok(keyboards)
    }
}

/// Set non-blocking mode on keyboard devices.
fn set_nonblocking(keyboards: &[Device]) -> Result<()> {
    for device in keyboards {
        let fd = device.as_raw_fd();
        let flags = fcntl(fd, FcntlArg::F_GETFL).context("Failed to get fd flags")?;
        let flags = OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK;
        fcntl(fd, FcntlArg::F_SETFL(flags)).context("Failed to set non-blocking")?;
    }
    Ok(())
}

/// Linux hotkey listener using evdev.
pub struct HotkeyListener {
    keyboards: Vec<Device>,
    hotkeys: Vec<Hotkey>,
}

impl HotkeyListener {
    /// Create a new listener with the given keyboards and hotkeys.
    pub fn new(keyboards: Vec<Device>, hotkeys: Vec<Hotkey>) -> Self {
        Self { keyboards, hotkeys }
    }

    /// Start listening for hotkeys in a background thread.
    /// Returns a receiver for hotkey events.
    pub fn start(self, running: Arc<AtomicBool>) -> Result<Receiver<HotkeyEvent>> {
        let (tx, rx) = mpsc::channel();
        set_nonblocking(&self.keyboards)?;
        start_keyboard_listener(self.keyboards, self.hotkeys, running, tx)?;
        Ok(rx)
    }
}

fn start_keyboard_listener(
    keyboards: Vec<Device>,
    hotkeys: Vec<Hotkey>,
    running: Arc<AtomicBool>,
    tx: Sender<HotkeyEvent>,
) -> Result<()> {
    // Convert hotkeys to evdev keys
    let evdev_hotkeys: Vec<(evdev::Key, Modifiers)> = hotkeys
        .iter()
        .map(|h| (to_evdev_key(h.key), h.modifiers))
        .collect();

    thread::spawn(move || {
        let mut keyboards = keyboards;
        let mut current_mods = Modifiers::default();
        let mut last_rescan = Instant::now();
        let mut had_error = false;

        // Minimum interval between keyboard rescans
        const RESCAN_INTERVAL: Duration = Duration::from_secs(10);

        while running.load(Ordering::Relaxed) {
            // Check if we need to rescan keyboards (after error and interval passed)
            if had_error && last_rescan.elapsed() >= RESCAN_INTERVAL {
                log::info!("Keyboard error detected, rescanning devices...");
                match find_keyboards() {
                    Ok(new_keyboards) => {
                        log::info!(
                            "Keyboards reconnected: found {} device(s)",
                            new_keyboards.len()
                        );
                        if set_nonblocking(&new_keyboards).is_ok() {
                            keyboards = new_keyboards;
                            current_mods = Modifiers::default();
                            had_error = false;
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to rescan keyboards: {}", e);
                    }
                }
                last_rescan = Instant::now();
            }

            let mut any_error = false;

            for device in keyboards.iter_mut() {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if let evdev::InputEventKind::Key(key) = event.kind() {
                                let pressed = event.value() == 1;
                                let released = event.value() == 0;

                                // Track modifier state
                                match key {
                                    evdev::Key::KEY_LEFTSHIFT | evdev::Key::KEY_RIGHTSHIFT => {
                                        current_mods.shift =
                                            pressed || (!released && current_mods.shift);
                                        if released {
                                            current_mods.shift = false;
                                        }
                                    }
                                    evdev::Key::KEY_LEFTCTRL | evdev::Key::KEY_RIGHTCTRL => {
                                        current_mods.ctrl =
                                            pressed || (!released && current_mods.ctrl);
                                        if released {
                                            current_mods.ctrl = false;
                                        }
                                    }
                                    evdev::Key::KEY_LEFTALT | evdev::Key::KEY_RIGHTALT => {
                                        current_mods.alt =
                                            pressed || (!released && current_mods.alt);
                                        if released {
                                            current_mods.alt = false;
                                        }
                                    }
                                    _ => {}
                                }

                                // Check each hotkey
                                for (idx, (hotkey_key, hotkey_mods)) in
                                    evdev_hotkeys.iter().enumerate()
                                {
                                    if key == *hotkey_key {
                                        let mods_match = current_mods.shift == hotkey_mods.shift
                                            && current_mods.ctrl == hotkey_mods.ctrl
                                            && current_mods.alt == hotkey_mods.alt;

                                        if mods_match {
                                            if pressed {
                                                let _ = tx.send(HotkeyEvent::Pressed(idx));
                                            } else if released {
                                                let _ = tx.send(HotkeyEvent::Released(idx));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // EAGAIN/EWOULDBLOCK is expected for non-blocking reads
                        if e.raw_os_error() != Some(libc::EAGAIN)
                            && e.raw_os_error() != Some(libc::EWOULDBLOCK)
                        {
                            log::debug!("Keyboard read error: {}", e);
                            any_error = true;
                        }
                    }
                }
            }

            if any_error {
                had_error = true;
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    Ok(())
}
