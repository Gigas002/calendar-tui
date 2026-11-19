use crate::date::{Date, WeekStart, Weekday, days_in_month, naive_from_ymd};

/// ISO week number for a grid row (week of the row's Thursday, per ISO-8601).
pub fn iso_week_number_for_row(week: &[DayCell; 7]) -> i8 {
    week.iter()
        .find(|c| c.date.weekday() == Weekday::Thursday)
        .or(week.first())
        .expect("week row has seven days")
        .date
        .iso_week_date()
        .week()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayCell {
    pub date: Date,
    pub day: i8,
    pub in_month: bool,
}

/// Maximum week rows a month grid can occupy (used for uniform year-view layout).
pub const MAX_WEEK_ROWS: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthGrid {
    pub year: i16,
    pub month: i8,
    pub weeks: Vec<[DayCell; 7]>,
}

impl MonthGrid {
    pub fn build(year: i16, month: i8, week_start: WeekStart) -> Self {
        let first = naive_from_ymd(year, month, 1).expect("valid month start");
        let len = days_in_month(year, month);
        let start_col = week_start.column_for(first.weekday()) as usize;

        let mut cells: Vec<DayCell> = Vec::new();

        for offset in (0..start_col).rev() {
            let date = first
                .checked_sub(jiff::Span::new().days(offset as i64 + 1))
                .expect("years are kept inside the date range (date::MIN_YEAR)");
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

        let mut tail = first.last_of_month();
        while !cells.is_empty() && !cells.len().is_multiple_of(7) {
            tail = tail
                .tomorrow()
                .expect("years are kept inside the date range (date::MAX_YEAR)");
            cells.push(DayCell {
                date: tail,
                day: tail.day(),
                in_month: false,
            });
        }

        let weeks = cells.as_chunks::<7>().0.to_vec();

        Self { year, month, weeks }
    }

    pub fn week_count(&self) -> usize {
        self.weeks.len()
    }

    pub fn iso_week_numbers(&self) -> Vec<i8> {
        self.weeks.iter().map(iso_week_number_for_row).collect()
    }
}

pub fn build_year_grids(year: i16, week_start: WeekStart) -> Vec<MonthGrid> {
    (1..=12)
        .map(|month| MonthGrid::build(year, month, week_start))
        .collect()
}

#[cfg(test)]
mod tests;
