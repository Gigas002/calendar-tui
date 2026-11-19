use std::path::{Path, PathBuf};

use ratatui::style::{Color, Style};
use serde::Deserialize;

use crate::config::config_dir;
use crate::error::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub base: Option<Base>,
    pub calendar: Option<Calendar>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Base {
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub border: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Calendar {
    pub header: Option<String>,
    pub status: Option<String>,
    pub today: Option<String>,
    pub selected: Option<String>,
    pub weekend: Option<String>,
    pub other_month: Option<String>,
}

/// Parsed `#RRGGBB` or `#RRGGBBAA` theme color.
///
/// Terminals do not blend alpha; `a == 0` leaves that layer unset (terminal shows through),
/// `a > 0` uses opaque RGB (alpha channel is not blended).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ThemeColor {
    pub const OPAQUE: u8 = 255;

    pub fn is_transparent(self) -> bool {
        self.a == 0
    }

    pub fn to_rgb(self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }

    pub fn patch_fg(self, style: Style) -> Style {
        if self.is_transparent() {
            style
        } else {
            style.fg(self.to_rgb())
        }
    }

    pub fn patch_bg(self, style: Style) -> Style {
        if self.is_transparent() {
            style
        } else {
            style.bg(self.to_rgb())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeColors {
    pub background: ThemeColor,
    pub foreground: ThemeColor,
    pub border: ThemeColor,
    pub header: ThemeColor,
    pub status: ThemeColor,
    pub today: ThemeColor,
    pub selected: ThemeColor,
    pub weekend: ThemeColor,
    pub other_month: ThemeColor,
}

impl ThemeColors {
    pub fn cell_base(&self) -> Style {
        let mut style = Style::default();
        style = self.foreground.patch_fg(style);
        style = self.background.patch_bg(style);
        style
    }
}

impl Default for Base {
    fn default() -> Self {
        Self {
            background: Some("#1e1e2eFF".to_string()),
            foreground: Some("#cdd6f4FF".to_string()),
            border: Some("#45475aFF".to_string()),
        }
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self {
            header: Some("#cba6f7FF".to_string()),
            status: Some("#a6e3a1FF".to_string()),
            today: Some("#f9e2afFF".to_string()),
            selected: Some("#89b4faFF".to_string()),
            weekend: Some("#f38ba8FF".to_string()),
            other_month: Some("#6c7086FF".to_string()),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            base: Some(Base::default()),
            calendar: Some(Calendar::default()),
        }
    }
}

impl Theme {
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
            Ok(theme) => theme,
            Err(err) => {
                tracing::warn!(%err, path = %path.display(), "theme unavailable, using defaults");
                Self::default()
            }
        }
    }

    pub fn resolve(&self) -> Result<ThemeColors, Error> {
        let base = self.base.clone().unwrap_or_default();
        let calendar = self.calendar.clone().unwrap_or_default();

        let color = |key: &str, value: Option<&str>, default: &str| {
            let value = value.unwrap_or(default);
            parse_color(value).map_err(|reason| Error::InvalidColor {
                key: key.to_string(),
                value: value.to_string(),
                reason,
            })
        };

        Ok(ThemeColors {
            background: color("base.background", base.background.as_deref(), "#1e1e2eFF")?,
            foreground: color("base.foreground", base.foreground.as_deref(), "#cdd6f4FF")?,
            border: color("base.border", base.border.as_deref(), "#45475aFF")?,
            header: color("calendar.header", calendar.header.as_deref(), "#cba6f7FF")?,
            status: color("calendar.status", calendar.status.as_deref(), "#a6e3a1FF")?,
            today: color("calendar.today", calendar.today.as_deref(), "#f9e2afFF")?,
            selected: color(
                "calendar.selected",
                calendar.selected.as_deref(),
                "#89b4faFF",
            )?,
            weekend: color("calendar.weekend", calendar.weekend.as_deref(), "#f38ba8FF")?,
            other_month: color(
                "calendar.other_month",
                calendar.other_month.as_deref(),
                "#6c7086FF",
            )?,
        })
    }
}

pub fn themes_dir() -> PathBuf {
    config_dir().join("themes")
}

pub fn resolve_path(config_path: &Path, theme: &str) -> PathBuf {
    let theme_path = Path::new(theme);
    if theme_path.is_absolute() {
        return theme_path.to_path_buf();
    }

    if let Some(parent) = config_path.parent() {
        let direct = parent.join(theme);
        if direct.is_file() {
            return direct;
        }
        let under_themes = parent.join("themes").join(theme);
        if under_themes.is_file() {
            return under_themes;
        }
    }

    let xdg = themes_dir().join(theme);
    if xdg.is_file() {
        return xdg;
    }

    themes_dir().join(theme)
}

/// Parses `#RRGGBB` or `#RRGGBBAA` (the `#` is optional); the error says what's wrong.
pub fn parse_color(value: &str) -> Result<ThemeColor, &'static str> {
    let hex = value.trim();
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    // Checked before slicing: a multi-byte character would put a slice boundary inside it,
    // and `from_str_radix` alone would accept a sign ("+f").
    if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("expected hex digits: #RRGGBB or #RRGGBBAA");
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("two hex digits");
    match hex.len() {
        6 => Ok(ThemeColor {
            r: byte(0),
            g: byte(2),
            b: byte(4),
            a: ThemeColor::OPAQUE,
        }),
        8 => Ok(ThemeColor {
            r: byte(0),
            g: byte(2),
            b: byte(4),
            a: byte(6),
        }),
        _ => Err("expected #RRGGBB or #RRGGBBAA"),
    }
}

#[cfg(test)]
mod tests;
