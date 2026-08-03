use crate::calendar::MonthGrid;
use crate::config::Config;
use crate::date::WeekStart;
use crate::settings::Settings;
use crate::theme::Theme;
use crate::view::ViewMode;

const EXAMPLE_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../examples/config.toml"
));
const EXAMPLE_THEME: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../examples/theme.toml"
));

#[test]
fn settings_from_examples() {
    let config: Config = toml::from_str(EXAMPLE_CONFIG).unwrap();
    let theme: Theme = toml::from_str(EXAMPLE_THEME).unwrap();
    let settings = Settings::resolve(&config, &theme).unwrap();
    assert!(settings.show_week_numbers);
    let expected_grid = MonthGrid::build(
        settings.view.view_year,
        settings.view.view_month,
        settings.week_start,
    );
    assert_eq!(settings.grid.week_count(), expected_grid.week_count());
    assert_eq!(settings.date_format, "%a, %d %b %Y");
    assert_eq!(settings.view.mode, ViewMode::Year);
}

#[test]
fn sunday_week_start_from_config_changes_grid_layout() {
    let config: Config = toml::from_str(
        r#"
[calendar]
week_start = "sunday"
"#,
    )
    .unwrap();
    let theme: Theme = toml::from_str(EXAMPLE_THEME).unwrap();
    let settings = Settings::resolve(&config, &theme).unwrap();
    assert_eq!(settings.week_start, WeekStart::Sunday);

    let monday_grid = MonthGrid::build(2026, 5, WeekStart::Monday);
    assert_ne!(settings.grid.weeks, monday_grid.weeks);
}
