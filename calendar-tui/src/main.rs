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
    let config = match Config::load_from(&config_path, cli.config.is_some()) {
        Ok(config) => config,
        Err(err) => {
            tracing::error!("{err}");
            return ExitCode::from(1);
        }
    };

    let theme_path = cli
        .theme
        .clone()
        .unwrap_or_else(|| theme::resolve_path(&config_path, config.theme_name()));
    let theme_named = cli.theme.is_some() || config.theme_is_set();
    let theme = match Theme::load_from(&theme_path, theme_named) {
        Ok(theme) => theme,
        Err(err) => {
            tracing::error!("{err}");
            return ExitCode::from(1);
        }
    };

    let settings = match Settings::from_cli(&cli, config, theme) {
        Ok(s) => s,
        Err(err) => {
            tracing::error!(%err, path = %theme_path.display(), "invalid theme");
            return ExitCode::from(1);
        }
    };

    tracing::debug!(
        week_start = ?settings.week_start,
        view_mode = ?settings.view.mode,
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
