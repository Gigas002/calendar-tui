use super::{Date, Weekday};

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
    assert_eq!(start.column_for(Weekday::Monday), 0);
    assert_eq!(start.column_for(Weekday::Sunday), 6);
}

#[test]
fn column_for_sunday_week_start() {
    let start = WeekStart::Sunday;
    assert_eq!(start.column_for(Weekday::Sunday), 0);
    assert_eq!(start.column_for(Weekday::Monday), 1);
    assert_eq!(start.column_for(Weekday::Saturday), 6);
}

#[test]
fn column_for_each_week_start_on_may_2026_first() {
    let first = Date::new(2026, 5, 1).unwrap();
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

#[test]
fn date_check_waits_until_midnight_at_most_a_minute() {
    use std::time::Duration;

    use jiff::tz::{Offset, TimeZone};

    use super::{MAX_DATE_CHECK_INTERVAL, until_date_check};

    let zone = TimeZone::fixed(Offset::constant(3));
    let at = |h, m, s| {
        jiff::civil::date(2026, 5, 18)
            .at(h, m, s, 0)
            .to_zoned_datetime(zone.clone())
            .unwrap()
    };
    assert_eq!(until_date_check(&at(23, 59, 30)), Duration::from_secs(30));
    assert_eq!(until_date_check(&at(23, 59, 59)), Duration::from_secs(1));
    assert_eq!(until_date_check(&at(10, 0, 0)), MAX_DATE_CHECK_INTERVAL);
    assert_eq!(until_date_check(&at(0, 0, 0)), MAX_DATE_CHECK_INTERVAL);
}
