use chrono::Datelike;

use super::ViewState;

#[test]
fn new_at_today_starts_without_selection_mode() {
    let view = ViewState::new_at_today();
    assert_eq!(view.view_year, view.today.year());
    assert_eq!(view.view_month, view.today.month());
    assert!(!view.selection_mode);
    assert_eq!(view.selected_day, None);
}

#[test]
fn clamp_selected_day_on_short_month() {
    let mut view = ViewState {
        view_year: 2026,
        view_month: 2,
        selected_day: Some(31),
        selection_mode: true,
        today: ViewState::new_at_today().today,
    };
    view.clamp_selected_day();
    assert_eq!(view.selected_day, Some(28));
}

#[test]
fn prev_month_from_march() {
    let mut view = ViewState {
        view_year: 2026,
        view_month: 3,
        selected_day: Some(15),
        selection_mode: true,
        today: ViewState::new_at_today().today,
    };
    view.prev_month();
    assert_eq!(view.view_year, 2026);
    assert_eq!(view.view_month, 2);
    assert_eq!(view.selected_day, Some(15));
}

#[test]
fn next_month_from_december_rolls_year() {
    let mut view = ViewState {
        view_year: 2026,
        view_month: 12,
        selected_day: Some(10),
        selection_mode: true,
        today: ViewState::new_at_today().today,
    };
    view.next_month();
    assert_eq!(view.view_year, 2027);
    assert_eq!(view.view_month, 1);
}

#[test]
fn prev_year_clamps_jan_31_to_feb() {
    let mut view = ViewState {
        view_year: 2026,
        view_month: 1,
        selected_day: Some(31),
        selection_mode: true,
        today: ViewState::new_at_today().today,
    };
    view.prev_year();
    assert_eq!(view.view_year, 2025);
    assert_eq!(view.selected_day, Some(31));
    view.next_month();
    assert_eq!(view.view_month, 2);
    assert_eq!(view.selected_day, Some(28));
}

#[test]
fn jump_to_today_resets_view() {
    let mut view = ViewState {
        view_year: 2020,
        view_month: 1,
        selected_day: None,
        selection_mode: false,
        today: ViewState::new_at_today().today,
    };
    view.jump_to_today();
    assert_eq!(view.view_year, view.today.year());
    assert_eq!(view.view_month, view.today.month());
    assert!(!view.selection_mode);
}

#[test]
fn toggle_selection_mode_on_and_off() {
    let today = ViewState::new_at_today().today;
    let mut view = ViewState {
        view_year: today.year(),
        view_month: today.month(),
        selected_day: None,
        selection_mode: false,
        today,
    };
    view.toggle_selection_mode();
    assert!(view.selection_mode);
    assert_eq!(view.selected_day, Some(today.day()));
    view.toggle_selection_mode();
    assert!(!view.selection_mode);
    assert_eq!(view.selected_day, None);
}

#[test]
fn toggle_selection_off_month_picks_first() {
    let today = ViewState::new_at_today().today;
    let mut view = ViewState {
        view_year: 2020,
        view_month: 6,
        selected_day: None,
        selection_mode: false,
        today,
    };
    view.toggle_selection_mode();
    assert_eq!(view.selected_day, Some(1));
}

#[test]
fn select_next_and_prev_day() {
    let mut view = ViewState {
        view_year: 2026,
        view_month: 5,
        selected_day: Some(10),
        selection_mode: true,
        today: ViewState::new_at_today().today,
    };
    view.select_next_day();
    assert_eq!(view.selected_day, Some(11));
    view.select_prev_day();
    assert_eq!(view.selected_day, Some(10));
    view.select_prev_day();
    assert_eq!(view.selected_day, Some(9));
}

#[test]
fn select_next_week_clamps_to_month_end() {
    let mut view = ViewState {
        view_year: 2026,
        view_month: 5,
        selected_day: Some(28),
        selection_mode: true,
        today: ViewState::new_at_today().today,
    };
    view.select_next_week();
    assert_eq!(view.selected_day, Some(31));
}
