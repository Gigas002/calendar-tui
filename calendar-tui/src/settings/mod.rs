use crate::calendar::{MonthGrid, build_year_grids};
use crate::cli::Cli;
use crate::config::Config;
use crate::date::WeekStart;
use crate::error::Error;
use crate::theme::{Theme, ThemeColors};
use crate::view::{ViewMode, ViewState};

#[derive(Debug, Clone)]
pub struct Settings {
    pub week_start: WeekStart,
    pub show_week_numbers: bool,
    pub date_format: String,
    pub month_year_format: String,
    pub colors: ThemeColors,
    pub view: ViewState,
    pub grid: MonthGrid,
    pub year_grids: Vec<MonthGrid>,
}

impl Settings {
    pub fn resolve(config: &Config, theme: &Theme) -> Result<Self, Error> {
        let colors = theme.resolve()?;
        let week_start = config.week_start();
        let view = ViewState::new_at_today(config.default_mode());
        let grid = MonthGrid::build(view.view_year, view.view_month, week_start);
        let year_grids = build_year_grids(view.view_year, week_start);

        let mut settings = Self {
            week_start,
            show_week_numbers: config.show_week_numbers(),
            date_format: config.date_format().to_string(),
            month_year_format: config.month_year_format().to_string(),
            colors,
            view,
            grid,
            year_grids,
        };
        settings.sync_view();
        Ok(settings)
    }

    pub fn from_cli(cli: &Cli, config: Config, theme: Theme) -> Result<Self, Error> {
        let _ = cli;
        Self::resolve(&config, &theme)
    }

    pub fn refresh_grid(&mut self) {
        self.grid = MonthGrid::build(self.view.view_year, self.view.view_month, self.week_start);
    }

    pub fn refresh_year_grids(&mut self) {
        self.year_grids = build_year_grids(self.view.view_year, self.week_start);
    }

    pub fn sync_view(&mut self) {
        self.view.clamp_selected_day();
        self.refresh_grid();
        if self.view.mode == ViewMode::Year {
            self.refresh_year_grids();
        }
    }
}

#[cfg(test)]
mod tests;
