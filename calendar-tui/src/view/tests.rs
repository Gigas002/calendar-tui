use chrono::Datelike;

use super::ViewState;

#[test]
fn new_at_today_has_selection() {
    let view = ViewState::new_at_today();
    assert_eq!(view.view_year, view.today.year());
    assert_eq!(view.view_month, view.today.month());
    assert_eq!(view.selected_day, Some(view.today.day()));
}

#[test]
fn clamp_selected_day_on_short_month() {
    let mut view = ViewState {
        view_year: 2026,
        view_month: 2,
        selected_day: Some(31),
        today: ViewState::new_at_today().today,
    };
    view.clamp_selected_day();
    assert_eq!(view.selected_day, Some(28));
}
