# hotkey-listener

Cross-platform global hotkey listener with native Wayland support.

## Features

- **Native Wayland support on Linux** - Uses evdev directly (reads `/dev/input`), not X11 APIs
- **Automatic keyboard reconnection** - Handles USB keyboard disconnect/reconnect gracefully
- **Modifier key support** - Parse and detect `Shift+F8` style hotkey combinations
- **Simple push-to-talk API** - Clean pressed/released event model
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
use hotkey_listener::{parse_hotkey, HotkeyListenerBuilder};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

fn main() -> anyhow::Result<()> {
    let hotkey = parse_hotkey("Shift+F8")?;

    let running = Arc::new(AtomicBool::new(true));
    let listener = HotkeyListenerBuilder::new()
        .add_hotkey(hotkey)
        .build()?;

    let rx = listener.start(running.clone());

    while let Ok(event) = rx.recv() {
        match event {
            hotkey_listener::HotkeyEvent::Pressed(idx) => {
                println!("Hotkey {} pressed", idx);
            }
            hotkey_listener::HotkeyEvent::Released(idx) => {
                println!("Hotkey {} released", idx);
            }
        }
    }

    Ok(())
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

## License

MIT License
