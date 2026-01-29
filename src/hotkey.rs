//! Hotkey definition with optional modifiers.

use crate::key::Key;
use anyhow::{anyhow, Result};

/// Modifier keys that can be combined with a hotkey.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// A hotkey consisting of a key and optional modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hotkey {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl Hotkey {
    /// Create a new hotkey with no modifiers.
    pub fn new(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::default(),
        }
    }

    /// Create a new hotkey with the given modifiers.
    pub fn with_modifiers(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Return a copy of this hotkey with the shift modifier added.
    pub fn with_shift(&self) -> Self {
        Self {
            key: self.key,
            modifiers: Modifiers {
                shift: true,
                ctrl: self.modifiers.ctrl,
                alt: self.modifiers.alt,
            },
        }
    }
}

impl std::fmt::Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if self.modifiers.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.modifiers.alt {
            parts.push("Alt".to_string());
        }
        if self.modifiers.shift {
            parts.push("Shift".to_string());
        }
        parts.push(self.key.to_string());
        write!(f, "{}", parts.join("+"))
    }
}

/// Parse a hotkey string like "Shift+F8" or "F10" into a Hotkey.
pub fn parse_hotkey(s: &str) -> Result<Hotkey> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = Modifiers::default();

    if parts.is_empty() {
        return Err(anyhow!("Empty hotkey string"));
    }

    // Parse modifiers (all parts except the last one)
    for part in &parts[..parts.len() - 1] {
        match part.to_uppercase().as_str() {
            "SHIFT" => modifiers.shift = true,
            "CTRL" | "CONTROL" => modifiers.ctrl = true,
            "ALT" => modifiers.alt = true,
            _ => return Err(anyhow!("Unknown modifier: {}", part)),
        }
    }

    // Parse the key (last part)
    let key_str = parts[parts.len() - 1];
    let key = Key::parse(key_str)?;

    Ok(Hotkey { key, modifiers })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_key() {
        let hotkey = parse_hotkey("F8").unwrap();
        assert_eq!(hotkey.key, Key::F8);
        assert!(!hotkey.modifiers.shift);
        assert!(!hotkey.modifiers.ctrl);
        assert!(!hotkey.modifiers.alt);
    }

    #[test]
    fn test_parse_with_shift() {
        let hotkey = parse_hotkey("Shift+F8").unwrap();
        assert_eq!(hotkey.key, Key::F8);
        assert!(hotkey.modifiers.shift);
        assert!(!hotkey.modifiers.ctrl);
        assert!(!hotkey.modifiers.alt);
    }

    #[test]
    fn test_parse_with_multiple_modifiers() {
        let hotkey = parse_hotkey("Ctrl+Alt+F1").unwrap();
        assert_eq!(hotkey.key, Key::F1);
        assert!(!hotkey.modifiers.shift);
        assert!(hotkey.modifiers.ctrl);
        assert!(hotkey.modifiers.alt);
    }

    #[test]
    fn test_parse_case_insensitive() {
        let hotkey = parse_hotkey("SHIFT+f8").unwrap();
        assert_eq!(hotkey.key, Key::F8);
        assert!(hotkey.modifiers.shift);
    }

    #[test]
    fn test_parse_unknown_key() {
        assert!(parse_hotkey("Unknown").is_err());
    }

    #[test]
    fn test_parse_unknown_modifier() {
        assert!(parse_hotkey("Meta+F8").is_err());
    }

    #[test]
    fn test_display() {
        let hotkey = parse_hotkey("Shift+F8").unwrap();
        assert_eq!(hotkey.to_string(), "Shift+F8");
    }
}
