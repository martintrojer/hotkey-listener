//! Platform-agnostic key representation.

use anyhow::{anyhow, Result};

/// Platform-agnostic key representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    ScrollLock,
    Pause,
    Insert,
}

impl Key {
    /// Parse a key from a string like "F8" or "ScrollLock".
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "F1" => Ok(Key::F1),
            "F2" => Ok(Key::F2),
            "F3" => Ok(Key::F3),
            "F4" => Ok(Key::F4),
            "F5" => Ok(Key::F5),
            "F6" => Ok(Key::F6),
            "F7" => Ok(Key::F7),
            "F8" => Ok(Key::F8),
            "F9" => Ok(Key::F9),
            "F10" => Ok(Key::F10),
            "F11" => Ok(Key::F11),
            "F12" => Ok(Key::F12),
            "SCROLLLOCK" | "SCROLL_LOCK" => Ok(Key::ScrollLock),
            "PAUSE" => Ok(Key::Pause),
            "INSERT" => Ok(Key::Insert),
            _ => Err(anyhow!("Unknown key: {}", s)),
        }
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::F1 => write!(f, "F1"),
            Key::F2 => write!(f, "F2"),
            Key::F3 => write!(f, "F3"),
            Key::F4 => write!(f, "F4"),
            Key::F5 => write!(f, "F5"),
            Key::F6 => write!(f, "F6"),
            Key::F7 => write!(f, "F7"),
            Key::F8 => write!(f, "F8"),
            Key::F9 => write!(f, "F9"),
            Key::F10 => write!(f, "F10"),
            Key::F11 => write!(f, "F11"),
            Key::F12 => write!(f, "F12"),
            Key::ScrollLock => write!(f, "ScrollLock"),
            Key::Pause => write!(f, "Pause"),
            Key::Insert => write!(f, "Insert"),
        }
    }
}
