use clap::Parser;

use super::Cli;

#[test]
fn cli_parses_config_and_theme_paths() {
    let cli = Cli::try_parse_from([
        "calendar-tui",
        "--config",
        "/tmp/config.toml",
        "--theme",
        "/tmp/theme.toml",
    ])
    .expect("cli should parse");

    assert_eq!(
        cli.config.as_deref().and_then(|p| p.to_str()),
        Some("/tmp/config.toml")
    );
    assert_eq!(
        cli.theme.as_deref().and_then(|p| p.to_str()),
        Some("/tmp/theme.toml")
    );
}
