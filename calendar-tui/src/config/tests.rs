use super::Config;

const EXAMPLE_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../examples/config.toml"
));

#[test]
fn example_config_toml_deserializes() {
    let cfg: Config = toml::from_str(EXAMPLE_CONFIG).expect("example config should parse");
    assert_eq!(cfg.week_start(), crate::date::WeekStart::Monday);
    assert!(!cfg.show_week_numbers());
    assert_eq!(cfg.theme_name(), "theme.toml");
    assert_eq!(cfg.date_format(), "%a, %d %b %Y");
    assert_eq!(cfg.month_year_format(), "%B %Y");
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
