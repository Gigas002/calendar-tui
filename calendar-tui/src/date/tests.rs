use chrono::{Datelike, NaiveDate, Weekday};

use super::{WeekStart, days_in_month, today_local};

#[test]
fn days_in_month_common_cases() {
    assert_eq!(days_in_month(2024, 2), 29);
    assert_eq!(days_in_month(2023, 2), 28);
    assert_eq!(days_in_month(2026, 5), 31);
    assert_eq!(days_in_month(2026, 4), 30);
}

#[test]
fn column_for_monday_week_start() {
    let start = WeekStart::Monday;
    assert_eq!(start.column_for(Weekday::Mon), 0);
    assert_eq!(start.column_for(Weekday::Sun), 6);
}

#[test]
fn column_for_sunday_week_start() {
    let start = WeekStart::Sunday;
    assert_eq!(start.column_for(Weekday::Sun), 0);
    assert_eq!(start.column_for(Weekday::Mon), 1);
    assert_eq!(start.column_for(Weekday::Sat), 6);
}

#[test]
fn column_for_each_week_start_on_may_2026_first() {
    let first = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let weekday = first.weekday();
    for start in WeekStart::ALL {
        let col = start.column_for(weekday);
        assert!(col < 7);
    }
}

#[test]
fn today_local_is_valid() {
    let today = today_local();
    assert!(today.year() >= 2020);
    assert!(today.month() >= 1 && today.month() <= 12);
}
