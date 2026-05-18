use chrono::Datelike;

use super::{ViewMode, ViewState};

fn month_view(
    view_year: i32,
    view_month: u32,
    selected_day: Option<u32>,
    selection_mode: bool,
) -> ViewState {
    ViewState {
        mode: ViewMode::Month,
        view_year,
        view_month,
        focused_month: view_month,
        selected_day,
        selection_mode,
        today: ViewState::new_at_today(ViewMode::Year).today,
    }
}

#[test]
fn new_at_today_starts_without_selection_mode() {
    let view = ViewState::new_at_today(ViewMode::Year);
    assert_eq!(view.view_year, view.today.year());
    assert_eq!(view.view_month, view.today.month());
    assert!(!view.selection_mode);
    assert_eq!(view.selected_day, None);
}

#[test]
fn clamp_selected_day_on_short_month() {
    let mut view = month_view(2026, 2, Some(31), true);
    view.clamp_selected_day();
    assert_eq!(view.selected_day, Some(28));
}

#[test]
fn prev_month_from_march() {
    let mut view = month_view(2026, 3, Some(15), true);
    view.prev_month();
    assert_eq!(view.view_year, 2026);
    assert_eq!(view.view_month, 2);
    assert_eq!(view.selected_day, Some(15));
}

#[test]
fn next_month_from_december_rolls_year() {
    let mut view = month_view(2026, 12, Some(10), true);
    view.next_month();
    assert_eq!(view.view_year, 2027);
    assert_eq!(view.view_month, 1);
}

#[test]
fn prev_year_clamps_jan_31_to_feb() {
    let mut view = month_view(2026, 1, Some(31), true);
    view.prev_year();
    assert_eq!(view.view_year, 2025);
    assert_eq!(view.selected_day, Some(31));
    view.next_month();
    assert_eq!(view.view_month, 2);
    assert_eq!(view.selected_day, Some(28));
}

#[test]
fn jump_to_today_resets_view() {
    let mut view = month_view(2020, 1, None, false);
    view.jump_to_today();
    assert_eq!(view.view_year, view.today.year());
    assert_eq!(view.view_month, view.today.month());
    assert!(!view.selection_mode);
}

#[test]
fn toggle_selection_mode_on_and_off() {
    let today = ViewState::new_at_today(ViewMode::Year).today;
    let mut view = month_view(today.year(), today.month(), None, false);
    view.toggle_selection_mode();
    assert!(view.selection_mode);
    assert_eq!(view.selected_day, Some(today.day()));
    view.toggle_selection_mode();
    assert!(!view.selection_mode);
    assert_eq!(view.selected_day, None);
}

#[test]
fn toggle_selection_off_month_picks_first() {
    let mut view = month_view(2020, 6, None, false);
    view.toggle_selection_mode();
    assert_eq!(view.selected_day, Some(1));
}

#[test]
fn select_next_and_prev_day() {
    let mut view = month_view(2026, 5, Some(10), true);
    view.select_next_day();
    assert_eq!(view.selected_day, Some(11));
    view.select_prev_day();
    assert_eq!(view.selected_day, Some(10));
    view.select_prev_day();
    assert_eq!(view.selected_day, Some(9));
}

#[test]
fn select_next_week_clamps_to_month_end() {
    let mut view = month_view(2026, 5, Some(28), true);
    view.select_next_week();
    assert_eq!(view.selected_day, Some(31));
}

#[test]
fn set_mode_year_syncs_focused_month() {
    let mut view = month_view(2026, 5, Some(10), true);
    view.set_mode(ViewMode::Year);
    assert_eq!(view.mode, ViewMode::Year);
    assert_eq!(view.focused_month, 5);
    assert!(!view.selection_mode);
    assert_eq!(view.selected_day, None);
}

#[test]
fn enter_month_mode_from_year_focus() {
    let mut view = month_view(2026, 3, None, false);
    view.set_mode(ViewMode::Year);
    view.focused_month = 8;
    view.enter_month_mode();
    assert_eq!(view.mode, ViewMode::Month);
    assert_eq!(view.view_month, 8);
}

#[test]
fn focus_prev_month_stops_at_january() {
    let mut view = month_view(2026, 1, None, false);
    view.set_mode(ViewMode::Year);
    view.focus_prev_month();
    assert_eq!(view.focused_month, 1);
}
