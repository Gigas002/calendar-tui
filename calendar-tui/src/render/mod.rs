use chrono::{Datelike, Weekday};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

mod year;

use crate::calendar::{DayCell, MonthGrid};
use crate::date::{WeekStart, naive_from_ymd};
use crate::settings::Settings;
use crate::theme::ThemeColors;
use crate::view::{ViewMode, ViewState};

const WEEKDAY_LABELS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
const HELP_MONTH_NAV: &str =
    "h/l month  j/k year  esc year-view  t today  space select  q quit";
const HELP_MONTH_SELECT: &str = "h/l day  k/j week  esc year-view  space exit  t today  q quit";
const HELP_YEAR: &str =
    "h/l month  j/k year  enter month-view  t today  q quit";

/// Draw the full calendar screen into `frame`.
pub fn draw(frame: &mut Frame, settings: &Settings) {
    match settings.view.mode {
        ViewMode::Month => draw_month(frame, settings),
        ViewMode::Year => year::draw_year(frame, settings),
    }
}

fn draw_month(frame: &mut Frame, settings: &Settings) {
    let area = frame.area();
    frame.render_widget(ratatui::widgets::Clear, area);
    if !settings.colors.background.is_transparent() {
        frame.render_widget(
            Paragraph::new("").style(settings.colors.background.patch_bg(Style::default())),
            area,
        );
    }

    let vertical = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .margin(1);

    let [
        status_area,
        header_area,
        weekdays_area,
        grid_area,
        help_area,
    ] = vertical.areas(area);

    draw_status(frame, status_area, settings);
    draw_header(frame, header_area, settings);
    draw_weekdays(frame, weekdays_area, settings);
    frame.render_widget(
        CalendarGrid {
            grid: &settings.grid,
            view: &settings.view,
            colors: &settings.colors,
        },
        grid_area,
    );
    draw_help(frame, help_area, settings);
}

pub fn weekday_labels(week_start: WeekStart) -> [String; 7] {
    let offset = week_start.days_from_monday() as usize;
    std::array::from_fn(|i| WEEKDAY_LABELS[(i + offset) % 7].to_string())
}

pub fn style_for_cell(cell: &DayCell, view: &ViewState, colors: &ThemeColors) -> Style {
    let base = colors.cell_base();

    if cell.date == view.today {
        return colors.today.patch_fg(base).add_modifier(Modifier::BOLD);
    }

    if !cell.in_month {
        return colors.other_month.patch_fg(base);
    }

    if view.selection_mode
        && cell.in_month
        && view.selected_day == Some(cell.day)
    {
        return colors
            .selected
            .patch_fg(base)
            .add_modifier(Modifier::REVERSED);
    }

    if matches!(cell.date.weekday(), Weekday::Sat | Weekday::Sun) {
        return colors.weekend.patch_fg(base);
    }

    base
}

/// Build a centered day label; underline applies only to the day digits when today.
#[cfg(test)]
pub fn day_line(
    cell: &DayCell,
    view: &ViewState,
    colors: &ThemeColors,
    width: usize,
) -> Line<'static> {
    let day_str = cell.day.to_string();
    let pad_len = width.saturating_sub(day_str.len());
    let pad = " ".repeat(pad_len);

    if cell.date == view.today {
        let pad_style = colors.foreground.patch_fg(colors.background.patch_bg(Style::default()));
        let day_style = colors
            .today
            .patch_fg(colors.background.patch_bg(Style::default()))
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        return Line::from(vec![
            Span::styled(pad, pad_style),
            Span::styled(day_str, day_style),
        ]);
    }

    Line::from(Span::styled(
        format!("{pad}{day_str}"),
        style_for_cell(cell, view, colors),
    ))
}

pub(crate) fn block_style(colors: &ThemeColors) -> Style {
    colors.background.patch_bg(Style::default())
}

fn draw_status(frame: &mut Frame, area: Rect, settings: &Settings) {
    let today_line = settings
        .view
        .today
        .format(&settings.date_format)
        .to_string();
    let text = format!("{}    Weeks: {}", today_line, settings.grid.week_count());

    let widget = Paragraph::new(text)
        .style(
            settings
                .colors
                .status
                .patch_fg(settings.colors.background.patch_bg(Style::default())),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(settings.colors.border.patch_fg(Style::default()))
                .style(block_style(&settings.colors))
                .title(" Today "),
        );
    frame.render_widget(widget, area);
}

fn draw_header(frame: &mut Frame, area: Rect, settings: &Settings) {
    let text = naive_from_ymd(settings.view.view_year, settings.view.view_month, 1)
        .map(|d| d.format(&settings.month_year_format).to_string())
        .unwrap_or_else(|| format!("{} / {}", settings.view.view_month, settings.view.view_year));

    let widget = Paragraph::new(text)
        .style(
            settings
                .colors
                .header
                .patch_fg(settings.colors.background.patch_bg(Style::default())),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(settings.colors.border.patch_fg(Style::default()))
                .style(block_style(&settings.colors)),
        );
    frame.render_widget(widget, area);
}

fn draw_weekdays(frame: &mut Frame, area: Rect, settings: &Settings) {
    let labels = weekday_labels(settings.week_start);
    let cols = column_rects(area, 7);
    let style = settings
        .colors
        .foreground
        .patch_fg(settings.colors.background.patch_bg(Style::default()))
        .add_modifier(Modifier::BOLD);
    for (label, col_area) in labels.iter().zip(cols) {
        let widget = Paragraph::new(label.as_str())
            .style(style)
            .alignment(Alignment::Center);
        frame.render_widget(widget, col_area);
    }
}

pub(crate) fn draw_help(frame: &mut Frame, area: Rect, settings: &Settings) {
    let help = match settings.view.mode {
        ViewMode::Year => HELP_YEAR,
        ViewMode::Month if settings.view.selection_mode => HELP_MONTH_SELECT,
        ViewMode::Month => HELP_MONTH_NAV,
    };
    let widget = Paragraph::new(help)
        .style(
            settings
                .colors
                .foreground
                .patch_fg(settings.colors.background.patch_bg(Style::default()))
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);
    frame.render_widget(widget, area);
}

fn split_axis(area: Rect, count: u16, horizontal: bool) -> Vec<Rect> {
    let count = count.max(1);
    let (total, pos) = if horizontal {
        (area.width, area.x)
    } else {
        (area.height, area.y)
    };
    let base = total / count;
    let extra = total % count;
    let mut offset = pos;
    (0..count)
        .map(|i| {
            let size = base + u16::from(i < extra);
            let rect = if horizontal {
                Rect {
                    x: offset,
                    y: area.y,
                    width: size,
                    height: area.height,
                }
            } else {
                Rect {
                    x: area.x,
                    y: offset,
                    width: area.width,
                    height: size,
                }
            };
            offset += size;
            rect
        })
        .collect()
}

pub(crate) fn column_rects(area: Rect, count: u16) -> Vec<Rect> {
    split_axis(area, count, true)
}

#[cfg(test)]
pub(crate) fn row_rects(area: Rect, count: u16) -> Vec<Rect> {
    split_axis(area, count, false)
}

struct CalendarGrid<'a> {
    grid: &'a MonthGrid,
    view: &'a ViewState,
    colors: &'a ThemeColors,
}

impl Widget for CalendarGrid<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.colors.border.patch_fg(Style::default()))
            .style(block_style(self.colors));
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let week_count = self.grid.weeks.len().max(1) as u16;
        let row_height = (inner.height / week_count).max(1);
        let cols = column_rects(inner, 7);

        for (row, week) in self.grid.weeks.iter().enumerate() {
            let row_y = inner.y.saturating_add(row as u16 * row_height);
            if row_y >= inner.y.saturating_add(inner.height) {
                break;
            }
            let row_area = Rect {
                x: inner.x,
                y: row_y,
                width: inner.width,
                height: row_height.min(inner.y.saturating_add(inner.height) - row_y),
            };

            for (col, cell) in week.iter().enumerate() {
                let col_area = cols.get(col).copied().unwrap_or(row_area);
                let cell_area = Rect {
                    x: col_area.x,
                    y: row_area.y,
                    width: col_area.width,
                    height: row_area.height,
                };
                if cell_area.width == 0 || cell_area.height == 0 {
                    continue;
                }
                render_day_cell(cell, self.view, self.colors, cell_area, buf);
            }
        }
    }
}

fn render_day_cell(
    cell: &DayCell,
    view: &ViewState,
    colors: &ThemeColors,
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
) {
    let day_str = cell.day.to_string();
    let line = if cell.date == view.today {
        let day_style = colors
            .today
            .patch_fg(colors.background.patch_bg(Style::default()))
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        Line::from(Span::styled(day_str, day_style))
    } else {
        Line::from(Span::styled(day_str, style_for_cell(cell, view, colors)))
    };

    let text_y = area.y + area.height.saturating_sub(1) / 2;
    let text_area = Rect {
        x: area.x,
        y: text_y,
        width: area.width,
        height: 1.min(area.height),
    };
    Paragraph::new(line)
        .alignment(Alignment::Center)
        .render(text_area, buf);
}

#[cfg(test)]
mod tests;
