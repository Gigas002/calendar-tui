mod app;
mod calendar;
mod cli;
mod config;
mod date;
mod error;
mod logger;
mod render;
mod settings;
mod theme;
mod view;

use std::process::ExitCode;

use clap::Parser;

use cli::Cli;
use config::{Config, default_config_path};
use error::Error;
use settings::Settings;
use theme::Theme;

fn main() -> ExitCode {
    logger::init();
    let cli = Cli::parse();

    let config_path = cli.config.clone().unwrap_or_else(default_config_path);
    let config = load_config(&config_path);

    let theme_name = config.theme_name();
    let theme_path = cli
        .theme
        .clone()
        .unwrap_or_else(|| theme::resolve_path(&config_path, theme_name));
    let theme = load_theme(&theme_path);

    let settings = match Settings::from_cli(&cli, config, theme) {
        Ok(s) => s,
        Err(err) => {
            tracing::error!(%err, "failed to resolve settings");
            return ExitCode::from(1);
        }
    };

    tracing::info!(
        week_start = ?settings.week_start,
        weeks = settings.grid.week_count(),
        view_month = settings.view.view_month,
        view_year = settings.view.view_year,
        "calendar-tui starting"
    );

    match app::run(settings) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::NotATty) => {
            tracing::error!("{}", Error::NotATty);
            ExitCode::from(2)
        }
        Err(err) => {
            tracing::error!(%err, "application error");
            ExitCode::from(1)
        }
    }
}

fn load_config(path: &std::path::Path) -> Config {
    if path.is_file() {
        match Config::load(path) {
            Ok(config) => config,
            Err(err) => {
                tracing::error!(%err, path = %path.display(), "invalid config");
                std::process::exit(1);
            }
        }
    } else {
        Config::load_or_default(path)
    }
}

fn load_theme(path: &std::path::Path) -> Theme {
    if path.is_file() {
        match Theme::load(path) {
            Ok(theme) => theme,
            Err(err) => {
                tracing::error!(%err, path = %path.display(), "invalid theme");
                std::process::exit(1);
            }
        }
    } else {
        Theme::load_or_default(path)
    }
}
