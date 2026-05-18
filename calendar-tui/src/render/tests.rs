use chrono::NaiveDate;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Modifier;

use ratatui::layout::Rect;

use super::{column_rects, day_line, draw, row_rects, style_for_cell, weekday_labels};
use crate::calendar::MAX_WEEK_ROWS;
use crate::calendar::{DayCell, MonthGrid, build_year_grids};
use crate::date::{WeekStart, naive_from_ymd};
use crate::settings::Settings;
use crate::theme::Theme;
use crate::view::{ViewMode, ViewState};

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
    settings.view.mode = ViewMode::Month;
    settings.view.focused_month = month;
    settings.view.selected_day = None;
    settings.view.selection_mode = false;
    settings.grid = MonthGrid::build(year, month, settings.week_start);
    settings
}

fn view_state(
    year: i32,
    month: u32,
    today: NaiveDate,
    selected_day: Option<u32>,
    selection_mode: bool,
) -> ViewState {
    ViewState {
        mode: ViewMode::Month,
        view_year: year,
        view_month: month,
        focused_month: month,
        selected_day,
        selection_mode,
        today,
    }
}

#[test]
fn column_rects_splits_width_evenly() {
    let area = Rect::new(0, 0, 20, 1);
    let cols = column_rects(area, 7);
    assert_eq!(cols.len(), 7);
    let total: u16 = cols.iter().map(|r| r.width).sum();
    assert_eq!(total, area.width);
    let min = cols.iter().map(|r| r.width).min().unwrap();
    let max = cols.iter().map(|r| r.width).max().unwrap();
    assert!(max - min <= 1, "column widths should differ by at most 1");
    assert_eq!(cols.last().unwrap().x + cols.last().unwrap().width, area.right());
}

#[test]
fn year_mini_week_rows_use_uniform_height_with_bottom_remainder() {
    let inner_height = 11u16;
    let slots = MAX_WEEK_ROWS as u16;
    let row_height = inner_height / slots;
    assert_eq!(row_height, 1);
    assert_eq!(row_height * slots, 6);
    assert_eq!(inner_height - row_height * slots, 5);
}

#[test]
fn row_rects_splits_height_evenly() {
    let area = Rect::new(0, 0, 1, 11);
    let rows = row_rects(area, 6);
    assert_eq!(rows.len(), 6);
    let total: u16 = rows.iter().map(|r| r.height).sum();
    assert_eq!(total, area.height);
    let min = rows.iter().map(|r| r.height).min().unwrap();
    let max = rows.iter().map(|r| r.height).max().unwrap();
    assert!(max - min <= 1);
    assert_eq!(rows.last().unwrap().y + rows.last().unwrap().height, area.bottom());
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
    let view = view_state(2026, 6, today, None, false);
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
    let view = view_state(2026, 5, today, None, false);
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
    let view = view_state(2026, 5, today, Some(18), true);
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

#[test]
fn selected_style_only_for_in_month_cells() {
    let theme: Theme = Theme::default();
    let colors = theme.resolve().unwrap();
    let today = naive_from_ymd(2026, 5, 18).unwrap();
    let view = view_state(2026, 5, today, Some(30), true);
    let trailing = DayCell {
        date: naive_from_ymd(2026, 6, 1).unwrap(),
        day: 1,
        in_month: false,
    };
    let style = style_for_cell(&trailing, &view, &colors);
    assert!(
        !style
            .add_modifier
            .contains(ratatui::style::Modifier::REVERSED)
    );
}

#[test]
fn draw_year_renders_all_month_titles() {
    let today = naive_from_ymd(2026, 5, 18).unwrap();
    let mut settings = example_settings_for_view(2026, 5, today);
    settings.view.mode = ViewMode::Year;
    settings.view.focused_month = 5;
    settings.year_grids = build_year_grids(2026, settings.week_start);

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| draw(f, &settings))
        .expect("draw should succeed");
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();
    for name in ["Jan", "Dec"] {
        assert!(content.contains(name), "missing month title {name}");
    }
}
