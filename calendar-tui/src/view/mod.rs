use chrono::{Datelike, NaiveDate};

use crate::date::{days_in_month, today_local};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewState {
    pub view_year: i32,
    pub view_month: u32,
    pub selected_day: Option<u32>,
    pub today: NaiveDate,
}

impl ViewState {
    pub fn new_at_today() -> Self {
        let today = today_local();
        let mut view = Self {
            view_year: today.year(),
            view_month: today.month(),
            selected_day: Some(today.day()),
            today,
        };
        view.clamp_selected_day();
        view
    }

    pub fn clamp_selected_day(&mut self) {
        let max = days_in_month(self.view_year, self.view_month);
        if let Some(day) = self.selected_day {
            if day > max {
                self.selected_day = Some(max);
            } else if day == 0 {
                self.selected_day = Some(1);
            }
        }
    }
}

#[cfg(test)]
mod tests;
