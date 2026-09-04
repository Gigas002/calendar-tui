use chrono::{Datelike, Duration, NaiveDate, Weekday};

use crate::date::{WeekStart, days_in_month, naive_from_ymd};

/// ISO week number for a grid row (week of the row's Thursday, per ISO-8601).
pub fn iso_week_number_for_row(week: &[DayCell; 7]) -> u32 {
    week.iter()
        .find(|c| c.date.weekday() == Weekday::Thu)
        .or(week.first())
        .expect("week row has seven days")
        .date
        .iso_week()
        .week()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayCell {
    pub date: NaiveDate,
    pub day: u32,
    pub in_month: bool,
}

/// Maximum week rows a month grid can occupy (used for uniform year-view layout).
pub const MAX_WEEK_ROWS: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthGrid {
    pub year: i32,
    pub month: u32,
    pub weeks: Vec<[DayCell; 7]>,
}

impl MonthGrid {
    pub fn build(year: i32, month: u32, week_start: WeekStart) -> Self {
        let first = naive_from_ymd(year, month, 1).expect("valid month start");
        let len = days_in_month(year, month);
        let last = naive_from_ymd(year, month, len).expect("valid month end");
        let start_col = week_start.column_for(first.weekday()) as usize;

        let mut cells: Vec<DayCell> = Vec::new();

        for offset in (0..start_col).rev() {
            let date = first - Duration::days((offset + 1) as i64);
            cells.push(DayCell {
                date,
                day: date.day(),
                in_month: false,
            });
        }

        for day in 1..=len {
            let date = naive_from_ymd(year, month, day).expect("valid in-month day");
            cells.push(DayCell {
                date,
                day,
                in_month: true,
            });
        }

        let mut tail = last + Duration::days(1);
        while !cells.is_empty() && !cells.len().is_multiple_of(7) {
            cells.push(DayCell {
                date: tail,
                day: tail.day(),
                in_month: false,
            });
            tail += Duration::days(1);
        }

        let weeks = cells.as_chunks::<7>().0.to_vec();

        Self { year, month, weeks }
    }

    pub fn week_count(&self) -> usize {
        self.weeks.len()
    }

    pub fn iso_week_numbers(&self) -> Vec<u32> {
        self.weeks.iter().map(iso_week_number_for_row).collect()
    }
}

pub fn build_year_grids(year: i32, week_start: WeekStart) -> Vec<MonthGrid> {
    (1..=12)
        .map(|month| MonthGrid::build(year, month, week_start))
        .collect()
}

#[cfg(test)]
mod tests;
