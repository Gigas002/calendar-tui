use crate::calendar::MonthGrid;
use crate::cli::Cli;
use crate::config::Config;
use crate::date::WeekStart;
use crate::error::Error;
use crate::theme::{Theme, ThemeColors};
use crate::view::ViewState;

#[derive(Debug, Clone)]
pub struct Settings {
    pub week_start: WeekStart,
    #[allow(dead_code)] // Phase 2+
    pub show_week_numbers: bool,
    pub date_format: String,
    pub month_year_format: String,
    pub colors: ThemeColors,
    pub view: ViewState,
    pub grid: MonthGrid,
}

impl Settings {
    pub fn resolve(config: &Config, theme: &Theme) -> Result<Self, Error> {
        let colors = theme.resolve()?;
        let week_start = config.week_start();
        let view = ViewState::new_at_today();
        let grid = MonthGrid::build(view.view_year, view.view_month, week_start);

        Ok(Self {
            week_start,
            show_week_numbers: config.show_week_numbers(),
            date_format: config.date_format().to_string(),
            month_year_format: config.month_year_format().to_string(),
            colors,
            view,
            grid,
        })
    }

    pub fn from_cli(cli: &Cli, config: Config, theme: Theme) -> Result<Self, Error> {
        let _ = cli;
        Self::resolve(&config, &theme)
    }
}

#[cfg(test)]
mod tests;
