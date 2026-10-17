use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::storage::{Loaded, data_dir, finish_rewrite, load, load_for_rewrite};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum KeyboardLayout {
    #[default]
    Qwerty,
    Dvorak,
    Colemak,
}

impl KeyboardLayout {
    pub fn cycle(self) -> Self {
        match self {
            Self::Qwerty => Self::Dvorak,
            Self::Dvorak => Self::Colemak,
            Self::Colemak => Self::Qwerty,
        }
    }
}

impl fmt::Display for KeyboardLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Qwerty => write!(f, "QWERTY"),
            Self::Dvorak => write!(f, "Dvorak"),
            Self::Colemak => write!(f, "Colemak"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyboardSettings {
    #[serde(default)]
    pub layout: KeyboardLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn cycle(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dark => write!(f, "dark"),
            Self::Light => write!(f, "light"),
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    #[serde(default = "default_true")]
    pub show_keyboard: bool,
    #[serde(default = "default_true")]
    pub show_hints: bool,
    #[serde(default = "default_true")]
    pub show_fingers: bool,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub include_punctuation: bool,
    #[serde(default)]
    pub include_numbers: bool,
    #[serde(default)]
    pub practice_mode: bool,
    #[serde(default)]
    pub error_stop: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            show_keyboard: true,
            show_hints: true,
            show_fingers: true,
            theme: Theme::default(),
            include_punctuation: false,
            include_numbers: false,
            practice_mode: false,
            error_stop: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub keyboard: KeyboardSettings,
    #[serde(default)]
    pub display: DisplaySettings,
}

fn settings_path() -> PathBuf {
    data_dir().join("settings.toml")
}

fn parse_settings(text: &str) -> Option<Settings> {
    toml::from_str(text).ok()
}

/// Loads the saved settings, or the defaults if there are none. `None` means the
/// file can't be parsed, and an error that it is there but can't be read.
pub fn load_settings() -> io::Result<Option<Settings>> {
    Ok(match load(&settings_path(), parse_settings)? {
        Loaded::Parsed(settings) => Some(settings),
        Loaded::Missing => Some(Settings::default()),
        Loaded::Unparseable => None,
    })
}

/// Saves `settings`, and returns a message saying where the old file was kept if
/// it couldn't be parsed.
pub fn save_settings(settings: &Settings) -> io::Result<Option<String>> {
    let path = settings_path();
    let (_, backup) = load_for_rewrite(&path, parse_settings)?;
    let content = toml::to_string_pretty(settings).map_err(io::Error::other)?;
    finish_rewrite(&path, &content, backup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn default_layout_is_qwerty() {
        let s = Settings::default();
        assert_eq!(s.keyboard.layout, KeyboardLayout::Qwerty);
    }

    #[test]
    fn cycle_layout() {
        assert_eq!(KeyboardLayout::Qwerty.cycle(), KeyboardLayout::Dvorak);
        assert_eq!(KeyboardLayout::Dvorak.cycle(), KeyboardLayout::Colemak);
        assert_eq!(KeyboardLayout::Colemak.cycle(), KeyboardLayout::Qwerty);
    }

    #[test]
    fn roundtrip_toml() {
        let s = Settings {
            keyboard: KeyboardSettings {
                layout: KeyboardLayout::Dvorak,
            },
            ..Settings::default()
        };
        let content = toml::to_string(&s).unwrap();
        let parsed: Settings = toml::from_str(&content).unwrap();
        assert_eq!(parsed.keyboard.layout, KeyboardLayout::Dvorak);
    }

    #[test]
    fn deserialize_empty_toml() {
        let parsed: Settings = toml::from_str("").unwrap();
        assert_eq!(parsed.keyboard.layout, KeyboardLayout::Qwerty);
    }

    #[test]
    fn display_names() {
        assert_eq!(KeyboardLayout::Qwerty.to_string(), "QWERTY");
        assert_eq!(KeyboardLayout::Dvorak.to_string(), "Dvorak");
        assert_eq!(KeyboardLayout::Colemak.to_string(), "Colemak");
    }

    #[test]
    fn default_theme_is_dark() {
        let s = Settings::default();
        assert_eq!(s.display.theme, Theme::Dark);
    }

    #[test]
    fn cycle_theme() {
        assert_eq!(Theme::Dark.cycle(), Theme::Light);
        assert_eq!(Theme::Light.cycle(), Theme::Dark);
    }

    #[test]
    fn theme_display() {
        assert_eq!(Theme::Dark.to_string(), "dark");
        assert_eq!(Theme::Light.to_string(), "light");
    }

    #[test]
    fn only_unreadable_settings_fail_to_load() {
        let dir = data_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(settings_path(), "[display").unwrap();
        let unparseable = load_settings();
        fs::remove_file(settings_path()).unwrap();
        fs::create_dir(settings_path()).unwrap(); // reading a folder fails, but not with "not found"
        let unreadable = load_settings();
        let _ = fs::remove_dir_all(&dir);

        assert!(matches!(unparseable, Ok(None)));
        assert!(unreadable.is_err());
    }
}
