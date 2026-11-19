use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::App;
use crate::config::Config;
use crate::date::naive_from_ymd;
use crate::settings::Settings;
use crate::theme::Theme;
use crate::view::{ViewMode, ViewState};

/// An app showing May 2026 in `mode`, with the 18th as today: no clock involved.
fn app_in(mode: ViewMode) -> App {
    let mut settings = Settings::resolve(&Config::default(), &Theme::default()).unwrap();
    settings.view = ViewState {
        mode,
        view_year: 2026,
        view_month: 5,
        focused_month: 5,
        selected_day: None,
        selection_mode: false,
        today: naive_from_ymd(2026, 5, 18).unwrap(),
    };
    settings.sync_view();
    App::new(settings)
}

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn view(app: &App) -> (ViewMode, i16, i8, i8) {
    let v = &app.settings.view;
    (v.mode, v.view_year, v.view_month, v.focused_month)
}

#[test]
fn q_and_ctrl_c_quit_from_every_mode() {
    for mode in [ViewMode::Month, ViewMode::Year] {
        let mut app = app_in(mode);
        press(&mut app, KeyCode::Char('q'));
        assert!(!app.running);

        let mut app = app_in(mode);
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!app.running);

        let mut app = app_in(mode);
        press(&mut app, KeyCode::Char('c'));
        assert!(app.running, "a plain c doesn't quit");
    }
}

#[test]
fn m_y_and_esc_switch_views() {
    let mut app = app_in(ViewMode::Year);
    press(&mut app, KeyCode::Char('m'));
    assert_eq!(view(&app).0, ViewMode::Month);
    press(&mut app, KeyCode::Esc);
    assert_eq!(view(&app).0, ViewMode::Year);
    press(&mut app, KeyCode::Esc);
    assert_eq!(view(&app).0, ViewMode::Year, "Esc in year view stays");
    press(&mut app, KeyCode::Char('m'));
    press(&mut app, KeyCode::Char('y'));
    assert_eq!(view(&app).0, ViewMode::Year);
}

#[test]
fn year_view_moves_the_focus_and_opens_months() {
    let mut app = app_in(ViewMode::Year);
    press(&mut app, KeyCode::Char('l'));
    press(&mut app, KeyCode::Right);
    assert_eq!(view(&app), (ViewMode::Year, 2026, 7, 7));
    press(&mut app, KeyCode::Char('h'));
    assert_eq!(view(&app), (ViewMode::Year, 2026, 6, 6));
    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::PageDown);
    assert_eq!(view(&app).1, 2028);
    assert_eq!(
        app.settings.year_grids[0].year, 2028,
        "grids follow the year"
    );
    press(&mut app, KeyCode::Char('k'));
    press(&mut app, KeyCode::PageUp);
    assert_eq!(view(&app).1, 2026);
    press(&mut app, KeyCode::Enter);
    assert_eq!(view(&app), (ViewMode::Month, 2026, 6, 6));
    assert_eq!((app.settings.grid.year, app.settings.grid.month), (2026, 6));
}

#[test]
fn month_view_moves_by_month_and_year_and_back_to_today() {
    let mut app = app_in(ViewMode::Month);
    press(&mut app, KeyCode::Char('l'));
    press(&mut app, KeyCode::PageDown);
    assert_eq!(view(&app), (ViewMode::Month, 2026, 7, 7));
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::PageUp);
    press(&mut app, KeyCode::Char('h'));
    assert_eq!(view(&app), (ViewMode::Month, 2026, 4, 4));
    press(&mut app, KeyCode::Down);
    assert_eq!(view(&app).1, 2027);
    press(&mut app, KeyCode::Char('k'));
    assert_eq!(view(&app).1, 2026);
    assert_eq!(app.settings.grid.month, 4, "grid follows the view");
}

#[test]
fn selection_mode_moves_by_day_and_week() {
    let mut app = app_in(ViewMode::Month);
    press(&mut app, KeyCode::Char(' '));
    assert!(app.settings.view.selection_mode);
    assert_eq!(app.settings.view.selected_day, Some(18), "starts on today");
    press(&mut app, KeyCode::Char('l'));
    press(&mut app, KeyCode::Down);
    assert_eq!(app.settings.view.selected_day, Some(26));
    press(&mut app, KeyCode::Char('h'));
    press(&mut app, KeyCode::PageUp);
    press(&mut app, KeyCode::Char('k'));
    assert_eq!(app.settings.view.selected_day, Some(11));
    press(&mut app, KeyCode::PageDown);
    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(
        app.settings.view.selected_day,
        Some(31),
        "clamped to the month"
    );
    assert_eq!(view(&app).2, 5, "selection doesn't change the month");
    press(&mut app, KeyCode::Enter);
    assert!(!app.settings.view.selection_mode);
    assert_eq!(app.settings.view.selected_day, None);
}

/// "Today" was set once at start-up, so a calendar left open past midnight kept highlighting
/// yesterday.
#[test]
fn set_today_moves_today_once() {
    let mut app = app_in(ViewMode::Year);
    let tomorrow = app.settings.view.today.tomorrow().unwrap();
    assert!(app.set_today(tomorrow));
    assert_eq!(app.settings.view.today, tomorrow);
    assert!(!app.set_today(tomorrow), "unchanged date: no redraw");
    let other = naive_from_ymd(2030, 1, 1).unwrap();
    assert!(app.set_today(other));
}
