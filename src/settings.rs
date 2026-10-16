use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

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

pub(crate) fn data_dir() -> PathBuf {
    if cfg!(test) {
        return std::env::temp_dir().join("clavirio-test-data");
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".clavirio")
}

/// Loads a data file that is about to be rewritten, using `parse`. A file that
/// can't be read or parsed is first renamed to a timestamped `.bak` file, so
/// neither the rewrite nor a later backup destroys it; `None` means that rename
/// failed and nothing should be written.
pub(crate) fn load_for_rewrite<T: Default>(
    path: &Path,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Option<T> {
    let loaded = match fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Some(T::default()),
        Err(_) => None,
    };
    if loaded.is_none() {
        let stamp = OffsetDateTime::now_utc().unix_timestamp();
        let name = path.file_name()?.to_string_lossy();
        fs::rename(path, path.with_file_name(format!("{name}.{stamp}.bak"))).ok()?;
    }
    Some(loaded.unwrap_or_default())
}

fn settings_path() -> PathBuf {
    data_dir().join("settings.toml")
}

pub fn load_settings() -> Settings {
    let path = settings_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &Settings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if load_for_rewrite(&path, |text| toml::from_str::<Settings>(text).ok()).is_none() {
        return;
    }
    if let Ok(content) = toml::to_string_pretty(settings) {
        let tmp = path.with_extension("toml.tmp");
        if fs::write(&tmp, &content).is_ok() {
            // fs::rename fails on Windows if destination exists; remove first
            let _ = fs::remove_file(&path);
            let _ = fs::rename(&tmp, &path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn unparseable_file_kept_before_rewrite() {
        let dir = std::env::temp_dir().join(format!("clavirio-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.toml");
        let parse = |text: &str| toml::from_str::<Settings>(text).ok();
        let broken = "[display]\ntheme = \"blue\"\n";

        let missing = load_for_rewrite(&path, parse);
        fs::write(&path, broken).unwrap();
        let unparseable = load_for_rewrite(&path, parse);
        let files_left: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|entry| fs::read_to_string(entry.ok()?.path()).ok())
            .collect();
        let _ = fs::remove_dir_all(&dir);

        assert!(missing.is_some());
        assert!(unparseable.is_some());
        assert_eq!(files_left, [broken]);
    }
}
