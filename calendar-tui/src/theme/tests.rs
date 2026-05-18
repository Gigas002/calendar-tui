use ratatui::style::Color;

use super::{Theme, parse_color};

const EXAMPLE_THEME: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../examples/theme.toml"
));

#[test]
fn example_theme_toml_deserializes() {
    let theme: Theme = toml::from_str(EXAMPLE_THEME).expect("example theme should parse");
    let colors = theme.resolve().expect("example colors should resolve");
    assert_eq!(colors.background, parse_color("#1e1e2eFF").unwrap());
    assert_eq!(colors.today, parse_color("#f9e2afFF").unwrap());
}

#[test]
fn parse_color_rgb_and_rgba() {
    assert_eq!(
        parse_color("#ff00ff").unwrap(),
        Color::from_u32(0x00ff_00ff)
    );
    assert_eq!(
        parse_color("#ff00ff80").unwrap(),
        Color::from_u32(0x80ff_00ff)
    );
}

#[test]
fn parse_color_rejects_bad_length() {
    assert!(parse_color("#fff").is_err());
}
