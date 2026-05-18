mod mode;

pub use mode::ViewMode;

use chrono::{Datelike, Months, NaiveDate};

use crate::date::{days_in_month, naive_from_ymd, today_local};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewState {
    pub mode: ViewMode,
    pub view_year: i32,
    pub view_month: u32,
    /// Month highlighted in year view (1–12); kept in sync with `view_month` when navigating.
    pub focused_month: u32,
    pub selected_day: Option<u32>,
    pub selection_mode: bool,
    pub today: NaiveDate,
}

impl ViewState {
    pub fn new_at_today(mode: ViewMode) -> Self {
        let today = today_local();
        let month = today.month();
        Self {
            mode,
            view_year: today.year(),
            view_month: month,
            focused_month: month,
            selected_day: None,
            selection_mode: false,
            today,
        }
    }

    pub fn set_mode(&mut self, mode: ViewMode) {
        if self.mode == mode {
            return;
        }
        self.selection_mode = false;
        self.selected_day = None;
        self.mode = mode;
        match mode {
            ViewMode::Month => {
                self.view_month = self.focused_month;
            }
            ViewMode::Year => {
                self.focused_month = self.view_month;
            }
        }
    }

    pub fn enter_month_mode(&mut self) {
        self.view_month = self.focused_month;
        self.set_mode(ViewMode::Month);
    }

    pub fn focus_prev_month(&mut self) {
        self.focused_month = self.focused_month.saturating_sub(1).max(1);
        self.view_month = self.focused_month;
    }

    pub fn focus_next_month(&mut self) {
        self.focused_month = (self.focused_month + 1).min(12);
        self.view_month = self.focused_month;
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

    pub fn toggle_selection_mode(&mut self) {
        if self.mode != ViewMode::Month {
            return;
        }
        if self.selection_mode {
            self.selection_mode = false;
            self.selected_day = None;
            return;
        }
        self.selection_mode = true;
        self.selected_day = Some(self.default_select_day());
    }

    pub fn select_prev_day(&mut self) {
        let Some(day) = self.selected_day else {
            return;
        };
        self.selected_day = Some(day.saturating_sub(1).max(1));
    }

    pub fn select_next_day(&mut self) {
        let Some(day) = self.selected_day else {
            return;
        };
        let max = days_in_month(self.view_year, self.view_month);
        self.selected_day = Some((day + 1).min(max));
    }

    pub fn select_prev_week(&mut self) {
        self.shift_selected_day(-7);
    }

    pub fn select_next_week(&mut self) {
        self.shift_selected_day(7);
    }

    pub fn prev_month(&mut self) {
        self.shift_month(-1);
        self.focused_month = self.view_month;
    }

    pub fn next_month(&mut self) {
        self.shift_month(1);
        self.focused_month = self.view_month;
    }

    pub fn prev_year(&mut self) {
        self.view_year -= 1;
        self.clamp_selected_day();
    }

    pub fn next_year(&mut self) {
        self.view_year += 1;
        self.clamp_selected_day();
    }

    pub fn jump_to_today(&mut self) {
        self.today = today_local();
        self.view_year = self.today.year();
        self.view_month = self.today.month();
        self.focused_month = self.today.month();
        if self.selection_mode && self.mode == ViewMode::Month {
            self.selected_day = Some(self.today.day());
        } else {
            self.selected_day = None;
        }
        self.clamp_selected_day();
    }

    fn default_select_day(&self) -> u32 {
        if self.view_year == self.today.year() && self.view_month == self.today.month() {
            self.today.day()
        } else {
            1
        }
    }

    fn shift_selected_day(&mut self, delta: i32) {
        let Some(day) = self.selected_day else {
            return;
        };
        let max = days_in_month(self.view_year, self.view_month) as i32;
        let new = (day as i32 + delta).clamp(1, max);
        self.selected_day = Some(new as u32);
    }

    fn shift_month(&mut self, delta: i32) {
        let Some(first) = naive_from_ymd(self.view_year, self.view_month, 1) else {
            return;
        };
        let months = Months::new(delta.unsigned_abs());
        let next = if delta < 0 {
            first.checked_sub_months(months)
        } else {
            first.checked_add_months(months)
        };
        let Some(next) = next else {
            return;
        };
        self.view_year = next.year();
        self.view_month = next.month();
        self.clamp_selected_day();
    }
}

#[cfg(test)]
mod tests;
