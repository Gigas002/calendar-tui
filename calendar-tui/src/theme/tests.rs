use ratatui::style::Style;

use super::{Theme, ThemeColor, parse_color};

const EXAMPLE_THEME: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../examples/theme.toml"
));

#[test]
fn example_theme_toml_deserializes() {
    let theme: Theme = toml::from_str(EXAMPLE_THEME).expect("example theme should parse");
    let colors = theme.resolve().expect("example colors should resolve");
    assert!(colors.background.is_transparent());
    assert_eq!(
        colors.foreground,
        ThemeColor {
            r: 0xcd,
            g: 0xd6,
            b: 0xf4,
            a: 255
        }
    );
    assert_eq!(
        colors.border,
        ThemeColor {
            r: 0x45,
            g: 0x47,
            b: 0x5a,
            a: 255
        }
    );
    assert_eq!(
        colors.header,
        ThemeColor {
            r: 0xcb,
            g: 0xa6,
            b: 0xf7,
            a: 255
        }
    );
    assert_eq!(
        colors.status,
        ThemeColor {
            r: 0xa6,
            g: 0xe3,
            b: 0xa1,
            a: 255
        }
    );
    assert_eq!(
        colors.today,
        ThemeColor {
            r: 0xf9,
            g: 0xe2,
            b: 0xaf,
            a: 255
        }
    );
    assert_eq!(
        colors.selected,
        ThemeColor {
            r: 0x89,
            g: 0xb4,
            b: 0xfa,
            a: 255
        }
    );
    assert_eq!(
        colors.weekend,
        ThemeColor {
            r: 0xf3,
            g: 0x8b,
            b: 0xa8,
            a: 255
        }
    );
    assert_eq!(
        colors.other_month,
        ThemeColor {
            r: 0x6c,
            g: 0x70,
            b: 0x86,
            a: 255
        }
    );
}

#[test]
fn partial_theme_uses_defaults_for_missing_keys() {
    let theme: Theme = toml::from_str(
        r##"
[base]
foreground = "#ffffffFF"

[calendar]
weekend = "#ff0000FF"
"##,
    )
    .unwrap();
    let colors = theme.resolve().unwrap();
    assert_eq!(
        colors.foreground,
        ThemeColor {
            r: 255,
            g: 255,
            b: 255,
            a: 255
        }
    );
    assert_eq!(
        colors.weekend,
        ThemeColor {
            r: 255,
            g: 0,
            b: 0,
            a: 255
        }
    );
    assert_eq!(colors.status.r, 0xa6);
    assert_eq!(colors.other_month.r, 0x6c);
}

#[test]
fn parse_color_rgb_and_rgba() {
    assert_eq!(
        parse_color("#ff00ff").unwrap(),
        ThemeColor {
            r: 255,
            g: 0,
            b: 255,
            a: 255
        }
    );
    assert_eq!(
        parse_color("#1e1e2e00").unwrap(),
        ThemeColor {
            r: 0x1e,
            g: 0x1e,
            b: 0x2e,
            a: 0
        }
    );
    assert_eq!(
        parse_color("#ff00ff80").unwrap(),
        ThemeColor {
            r: 255,
            g: 0,
            b: 255,
            a: 128
        }
    );
}

#[test]
fn transparent_color_omits_background() {
    let c = parse_color("#1e1e2e00").unwrap();
    let style = c.patch_bg(Style::default());
    assert_eq!(style.bg, None);
}

#[test]
fn opaque_color_sets_background() {
    let c = parse_color("#1e1e2eFF").unwrap();
    let style = c.patch_bg(Style::default());
    assert_eq!(style.bg, Some(c.to_rgb()));
}

#[test]
fn parse_color_rejects_bad_length() {
    assert!(parse_color("#fff").is_err());
}
