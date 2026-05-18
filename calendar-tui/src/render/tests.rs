use chrono::{Datelike, NaiveDate};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Modifier;

use super::{day_line, draw, style_for_cell, weekday_labels};
use crate::calendar::{DayCell, MonthGrid};
use crate::date::{WeekStart, naive_from_ymd};
use crate::settings::Settings;
use crate::theme::Theme;
use crate::view::ViewState;

fn example_settings_for_view(year: i32, month: u32, today: NaiveDate) -> Settings {
    let config: crate::config::Config = toml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/config.toml"
    )))
    .unwrap();
    let theme: Theme = toml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/theme.toml"
    )))
    .unwrap();
    let mut settings = Settings::resolve(&config, &theme).unwrap();
    settings.view.view_year = year;
    settings.view.view_month = month;
    settings.view.today = today;
    settings.view.selected_day = Some(today.day());
    settings.grid = MonthGrid::build(year, month, settings.week_start);
    settings
}

#[test]
fn weekday_labels_rotate_with_week_start() {
    assert_eq!(weekday_labels(WeekStart::Monday)[0], "Mo");
    assert_eq!(weekday_labels(WeekStart::Sunday)[0], "Su");
}

#[test]
fn style_for_cell_highlights_today_in_adjacent_month() {
    let theme: Theme = Theme::default();
    let colors = theme.resolve().unwrap();
    let today = naive_from_ymd(2026, 5, 18).unwrap();
    let view = ViewState {
        view_year: 2026,
        view_month: 6,
        selected_day: None,
        today,
    };
    let cell = DayCell {
        date: today,
        day: 18,
        in_month: false,
    };
    let style = style_for_cell(&cell, &view, &colors);
    assert!(style.add_modifier.contains(Modifier::BOLD));
    assert_eq!(style.fg, Some(colors.today.to_rgb()));
}

#[test]
fn style_for_cell_uses_other_month_when_not_today() {
    let theme: Theme = Theme::default();
    let colors = theme.resolve().unwrap();
    let today = naive_from_ymd(2026, 5, 18).unwrap();
    let view = ViewState {
        view_year: 2026,
        view_month: 5,
        selected_day: None,
        today,
    };
    let cell = DayCell {
        date: naive_from_ymd(2026, 4, 30).unwrap(),
        day: 30,
        in_month: false,
    };
    let style = style_for_cell(&cell, &view, &colors);
    assert_eq!(style.fg, Some(colors.other_month.to_rgb()));
}

#[test]
fn draw_renders_today_digit_in_view_month() {
    let today = naive_from_ymd(2026, 5, 18).unwrap();
    let settings = example_settings_for_view(2026, 5, today);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| draw(f, &settings))
        .expect("draw should succeed");
    let buf = terminal.backend().buffer();
    let content: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(content.contains('1'));
    assert!(content.contains('8'));
}

#[test]
fn today_underline_only_on_day_digits() {
    let theme: Theme = Theme::default();
    let colors = theme.resolve().unwrap();
    let today = naive_from_ymd(2026, 5, 18).unwrap();
    let view = ViewState {
        view_year: 2026,
        view_month: 5,
        selected_day: Some(18),
        today,
    };
    let cell = DayCell {
        date: today,
        day: 18,
        in_month: true,
    };
    let line = day_line(&cell, &view, &colors, 5);
    let spans: Vec<_> = line.iter().collect();
    assert_eq!(spans.len(), 2);
    assert!(!spans[0].style.add_modifier.contains(Modifier::UNDERLINED));
    assert!(spans[1].style.add_modifier.contains(Modifier::UNDERLINED));
    assert_eq!(spans[1].content, "18");
}

#[test]
fn today_style_requires_matching_date_not_day_number() {
    let today = naive_from_ymd(2026, 5, 18).unwrap();
    let settings = example_settings_for_view(2026, 6, today);
    let june_18 = DayCell {
        date: naive_from_ymd(2026, 6, 18).unwrap(),
        day: 18,
        in_month: true,
    };
    let style = style_for_cell(&june_18, &settings.view, &settings.colors);
    assert_ne!(style.fg, Some(settings.colors.today.to_rgb()));

    let may_18 = DayCell {
        date: today,
        day: 18,
        in_month: true,
    };
    let style = style_for_cell(&may_18, &settings.view, &settings.colors);
    assert_eq!(style.fg, Some(settings.colors.today.to_rgb()));
}
