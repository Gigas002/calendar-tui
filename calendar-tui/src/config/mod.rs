use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::date::WeekStart;
use crate::error::Error;
use crate::view::ViewMode;

/// The config file as written: every key optional. The getters below supply the defaults, so
/// `Config::default()` (no file) and an empty file behave the same.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub calendar: Option<Calendar>,
    pub display: Option<Display>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Calendar {
    pub week_start: Option<WeekStart>,
    pub show_week_numbers: Option<bool>,
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Display {
    pub date_format: Option<String>,
    pub month_year_format: Option<String>,
    /// Starting view: `month` or `year` (also toggled with `m` / `y` at runtime).
    pub default_mode: Option<ViewMode>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let raw = std::fs::read_to_string(path).map_err(|source| Error::ReadConfig {
            path: path.display().to_string(),
            source,
        })?;
        toml::from_str(&raw).map_err(|source| Error::TomlParse {
            path: path.display().to_string(),
            source,
        })
    }

    /// Loads `path` if it's there. A file the user named (`required`) must exist; the default
    /// location may be missing, which means the defaults.
    pub fn load_from(path: &Path, required: bool) -> Result<Self, Error> {
        if !required && !path.exists() {
            tracing::debug!(path = %path.display(), "no config file, using defaults");
            return Ok(Self::default());
        }
        Self::load(path)
    }

    pub fn week_start(&self) -> WeekStart {
        self.calendar
            .as_ref()
            .and_then(|c| c.week_start)
            .unwrap_or(WeekStart::Monday)
    }

    pub fn show_week_numbers(&self) -> bool {
        self.calendar
            .as_ref()
            .and_then(|c| c.show_week_numbers)
            .unwrap_or(false)
    }

    /// Whether the config names a theme (rather than falling back to `theme.toml`).
    pub fn theme_is_set(&self) -> bool {
        self.calendar.as_ref().is_some_and(|c| c.theme.is_some())
    }

    pub fn theme_name(&self) -> &str {
        self.calendar
            .as_ref()
            .and_then(|c| c.theme.as_deref())
            .unwrap_or("theme.toml")
    }

    pub fn date_format(&self) -> &str {
        self.display
            .as_ref()
            .and_then(|d| d.date_format.as_deref())
            .unwrap_or("%a, %d %b %Y")
    }

    pub fn month_year_format(&self) -> &str {
        self.display
            .as_ref()
            .and_then(|d| d.month_year_format.as_deref())
            .unwrap_or("%B %Y")
    }

    pub fn default_mode(&self) -> ViewMode {
        self.display
            .as_ref()
            .and_then(|d| d.default_mode)
            .unwrap_or(ViewMode::Year)
    }
}

pub fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("calendar-tui")
}

pub fn default_config_path() -> PathBuf {
    config_dir().join("config.toml")
}

#[cfg(test)]
mod tests;
