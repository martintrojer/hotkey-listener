# hotkey-listener

Cross-platform global hotkey listener with native Wayland support.

## Features

- **Native Wayland support on Linux** - Uses evdev directly (reads `/dev/input`), not X11 APIs
- **Automatic keyboard reconnection** - Handles USB keyboard disconnect/reconnect gracefully
- **Modifier key support** - Parse and detect `Shift+F8` style hotkey combinations
- **Simple push-to-talk API** - Clean pressed/released event model
- **Automatic cleanup** - Background thread stops when handle is dropped
- **Cross-platform** - Linux (evdev) + macOS (rdev) with unified API

## Why This Crate?

Most existing global hotkey crates for Rust rely on X11 APIs on Linux, which don't work on Wayland. This crate uses evdev to read directly from `/dev/input`, making it compatible with both X11 and Wayland.

| Crate | Linux Wayland | macOS | Keyboard Reconnection |
|-------|---------------|-------|----------------------|
| `hotkey-listener` | Native | Yes | Yes |
| `global-hotkey` | X11 only | Yes | No |
| `livesplit-hotkey` | Partial | Yes | No |
| `rdev` | X11 only* | Yes | No |

*rdev's `unstable_grab` feature works on Wayland but requires root.

## Usage

```rust
use hotkey_listener::{parse_hotkey, HotkeyListenerBuilder, HotkeyEvent};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let hotkey = parse_hotkey("Shift+F8")?;

    // Build and start the listener
    let handle = HotkeyListenerBuilder::new()
        .add_hotkey(hotkey)
        .build()?
        .start()?;

    // Receive hotkey events
    loop {
        match handle.recv_timeout(Duration::from_millis(100)) {
            Ok(HotkeyEvent::Pressed(idx)) => {
                println!("Hotkey {} pressed", idx);
            }
            Ok(HotkeyEvent::Released(idx)) => {
                println!("Hotkey {} released", idx);
            }
            Err(_) => {
                // Timeout - check for exit conditions, do other work, etc.
            }
        }
    }

    // Background thread stops automatically when `handle` is dropped
}
```

## Supported Keys

Function keys: `F1` through `F12`
Special keys: `ScrollLock`, `Pause`, `Insert`
Modifiers: `Shift`, `Ctrl`, `Alt`

## Linux Requirements

On Linux, the user must have permission to read from `/dev/input/event*` devices. This typically means:

- Running as root, or
- Being a member of the `input` group: `sudo usermod -aG input $USER`

## Platform Notes

### Linux
The listener thread polls `/dev/input` devices and responds immediately when the handle is dropped.

### macOS
The listener uses `rdev::listen()` which receives **all** keyboard events system-wide (not just registered hotkeys) and filters them. Due to limitations in `rdev`, the listener thread cannot be interrupted once started - it will only terminate when the process exits. This is generally fine since handle cleanup typically occurs at program shutdown.

## License

MIT License
