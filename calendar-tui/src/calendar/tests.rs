use super::MonthGrid;
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
        assert!(grid.week_count() >= 5 && grid.week_count() <= 6);
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

fn in_month_days(grid: &MonthGrid) -> u32 {
    grid.weeks
        .iter()
        .flat_map(|w| w.iter())
        .filter(|c| c.in_month)
        .count() as u32
}
