use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::date::WeekStart;
use crate::error::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub calendar: Option<Calendar>,
    pub display: Option<Display>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Calendar {
    pub week_start: Option<WeekStart>,
    pub show_week_numbers: Option<bool>,
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Display {
    pub date_format: Option<String>,
    pub month_year_format: Option<String>,
}

impl Default for Calendar {
    fn default() -> Self {
        Self {
            week_start: Some(WeekStart::Monday),
            show_week_numbers: Some(false),
            theme: Some("theme.toml".to_string()),
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self {
            date_format: Some("%a, %d %b %Y".to_string()),
            month_year_format: Some("%B %Y".to_string()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            calendar: Some(Calendar::default()),
            display: Some(Display::default()),
        }
    }
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

    pub fn load_or_default(path: &Path) -> Self {
        match Self::load(path) {
            Ok(config) => config,
            Err(err) => {
                tracing::warn!(%err, path = %path.display(), "config unavailable, using defaults");
                Self::default()
            }
        }
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
