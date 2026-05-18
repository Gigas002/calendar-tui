use crate::config::Config;
use crate::settings::Settings;
use crate::theme::Theme;

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
    assert_eq!(settings.grid.week_count(), 5);
    assert_eq!(settings.date_format, "%a, %d %b %Y");
}
