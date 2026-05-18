use std::path::Path;

use super::Config;
use crate::theme::{Theme, resolve_path};
use crate::view::ViewMode;

const EXAMPLE_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../examples/config.toml"
));

fn example_config_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/config.toml")
}

fn example_theme_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/theme.toml")
}

#[test]
fn example_config_toml_deserializes() {
    let cfg: Config = toml::from_str(EXAMPLE_CONFIG).expect("example config should parse");
    assert_eq!(cfg.week_start(), crate::date::WeekStart::Monday);
    assert!(cfg.show_week_numbers());
    assert_eq!(cfg.theme_name(), "theme.toml");
    assert_eq!(cfg.date_format(), "%a, %d %b %Y");
    assert_eq!(cfg.month_year_format(), "%B %Y");
    assert_eq!(cfg.default_mode(), ViewMode::Year);
}

#[test]
fn invalid_week_start_rejected() {
    let err = toml::from_str::<Config>(
        r#"
[calendar]
week_start = "funday"
"#,
    )
    .expect_err("unknown week_start should fail");
    assert!(err.to_string().contains("week_start") || err.to_string().contains("unknown"));
}

#[test]
fn invalid_default_mode_rejected() {
    let err = toml::from_str::<Config>(
        r#"
[display]
default_mode = "week"
"#,
    )
    .expect_err("unknown default_mode should fail");
    assert!(err.to_string().contains("default_mode") || err.to_string().contains("unknown"));
}

#[test]
fn sunday_week_start_from_config() {
    let cfg: Config = toml::from_str(
        r#"
[calendar]
week_start = "sunday"
"#,
    )
    .unwrap();
    assert_eq!(cfg.week_start(), crate::date::WeekStart::Sunday);
}

#[test]
fn example_config_file_on_disk_loads() {
    let path = example_config_path();
    assert!(path.is_file(), "missing {}", path.display());
    Config::load(&path).expect("examples/config.toml should load");
}

#[test]
fn example_config_theme_resolves_to_example_theme_file() {
    let config_path = example_config_path();
    let cfg = Config::load(&config_path).unwrap();
    let theme_path = resolve_path(&config_path, cfg.theme_name());
    assert_eq!(
        theme_path,
        example_theme_path(),
        "theme should resolve next to examples/config.toml"
    );
    Theme::load(&theme_path).expect("examples/theme.toml should parse");
}

#[test]
fn config_without_display_defaults_to_year_mode() {
    let cfg: Config = toml::from_str(
        r#"
[calendar]
week_start = "monday"
"#,
    )
    .unwrap();
    assert_eq!(cfg.default_mode(), ViewMode::Year);
}
