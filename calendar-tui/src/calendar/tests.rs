use chrono::{Datelike, Weekday};

use super::{MAX_WEEK_ROWS, MonthGrid, build_year_grids};
use crate::date::WeekStart;

#[test]
fn february_2024_has_five_weeks_monday_start() {
    let grid = MonthGrid::build(2024, 2, WeekStart::Monday);
    assert_eq!(grid.week_count(), 5);
    assert_eq!(in_month_days(&grid), 29);
}

#[test]
fn february_2023_has_five_weeks_monday_start() {
    let grid = MonthGrid::build(2023, 2, WeekStart::Monday);
    assert_eq!(grid.week_count(), 5);
    assert_eq!(in_month_days(&grid), 28);
}

#[test]
fn may_2026_has_five_weeks_monday_start() {
    let grid = MonthGrid::build(2026, 5, WeekStart::Monday);
    assert_eq!(grid.week_count(), 5);
    assert_eq!(in_month_days(&grid), 31);
}

#[test]
fn each_week_start_produces_valid_grid_for_may_2026() {
    for start in WeekStart::ALL {
        let grid = MonthGrid::build(2026, 5, start);
        assert!(grid.week_count() >= 5 && grid.week_count() <= MAX_WEEK_ROWS);
        assert_eq!(grid.weeks.len(), grid.week_count());
        assert_eq!(in_month_days(&grid), 31);
        for week in &grid.weeks {
            assert_eq!(week.len(), 7);
        }
    }
}

#[test]
fn sunday_start_can_add_a_row_vs_monday_for_same_month() {
    let monday = MonthGrid::build(2026, 8, WeekStart::Monday);
    let sunday = MonthGrid::build(2026, 8, WeekStart::Sunday);
    assert!(sunday.week_count() >= monday.week_count());
}

#[test]
fn first_row_starts_with_leading_days_when_month_not_on_week_start() {
    let grid = MonthGrid::build(2026, 5, WeekStart::Monday);
    let first = &grid.weeks[0];
    assert!(!first[0].in_month);
    assert!(first[6].in_month);
}

#[test]
fn grid_cells_are_contiguous_dates() {
    let grid = MonthGrid::build(2026, 5, WeekStart::Monday);
    let flat: Vec<_> = grid.weeks.iter().flat_map(|w| w.iter()).collect();
    for window in flat.windows(2) {
        assert_eq!(
            window[1]
                .date
                .signed_duration_since(window[0].date)
                .num_days(),
            1
        );
    }
}

#[test]
fn year_grids_never_exceed_max_week_rows() {
    for start in WeekStart::ALL {
        let grids = build_year_grids(2026, start);
        for grid in &grids {
            assert!(
                grid.week_count() <= MAX_WEEK_ROWS,
                "month {} has {} weeks",
                grid.month,
                grid.week_count()
            );
        }
    }
}

#[test]
fn monday_start_first_column_is_monday() {
    let grid = MonthGrid::build(2026, 5, WeekStart::Monday);
    assert_eq!(grid.weeks[0][0].date.weekday(), Weekday::Mon);
}

#[test]
fn sunday_start_first_column_is_sunday() {
    let grid = MonthGrid::build(2026, 5, WeekStart::Sunday);
    assert_eq!(grid.weeks[0][0].date.weekday(), Weekday::Sun);
}

#[test]
fn may_2026_first_in_month_day_column_differs_by_week_start() {
    let monday = MonthGrid::build(2026, 5, WeekStart::Monday);
    let sunday = MonthGrid::build(2026, 5, WeekStart::Sunday);
    assert_eq!(column_for_in_month_day(&monday, 1), Some(4));
    assert_eq!(column_for_in_month_day(&sunday, 1), Some(5));
}

#[test]
fn build_year_grids_has_twelve_months() {
    let grids = build_year_grids(2026, WeekStart::Monday);
    assert_eq!(grids.len(), 12);
    assert_eq!(grids[0].month, 1);
    assert_eq!(grids[11].month, 12);
    assert_eq!(grids[4].month, 5);
}

fn column_for_in_month_day(grid: &MonthGrid, day: u32) -> Option<usize> {
    for week in &grid.weeks {
        for (col, cell) in week.iter().enumerate() {
            if cell.in_month && cell.day == day {
                return Some(col);
            }
        }
    }
    None
}

fn in_month_days(grid: &MonthGrid) -> u32 {
    grid.weeks
        .iter()
        .flat_map(|w| w.iter())
        .filter(|c| c.in_month)
        .count() as u32
}

#[test]
fn iso_week_numbers_match_thursday_in_each_row() {
    let grid = MonthGrid::build(2026, 5, WeekStart::Monday);
    let numbers = grid.iso_week_numbers();
    assert_eq!(numbers.len(), grid.week_count());
    for (week, iso_week) in grid.weeks.iter().zip(numbers) {
        let thursday = week
            .iter()
            .find(|c| c.date.weekday() == Weekday::Thu)
            .expect("each week row contains a Thursday");
        assert_eq!(iso_week, thursday.date.iso_week().week());
    }
}
