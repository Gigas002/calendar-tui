use chrono::{Datelike, Weekday};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::calendar::{DayCell, MonthGrid};
use crate::date::{WeekStart, naive_from_ymd};
use crate::settings::Settings;
use crate::theme::ThemeColors;
use crate::view::ViewState;

const WEEKDAY_LABELS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
const HELP: &str = "q quit";

/// Draw the full calendar screen into `frame`.
pub fn draw(frame: &mut Frame, settings: &Settings) {
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
        Constraint::Min(1),
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
    draw_help(frame, help_area, &settings.colors);
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

    if view.selected_day == Some(cell.day) {
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

/// Build a right-aligned day label; underline applies only to the day digits when today.
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

fn block_style(colors: &ThemeColors) -> Style {
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

fn draw_help(frame: &mut Frame, area: Rect, colors: &ThemeColors) {
    let widget = Paragraph::new(HELP)
        .style(
            colors
                .foreground
                .patch_fg(colors.background.patch_bg(Style::default()))
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);
    frame.render_widget(widget, area);
}

fn column_rects(area: Rect, count: u16) -> Vec<Rect> {
    let count = count.max(1);
    let col_width = area.width / count;
    (0..count)
        .map(|i| Rect {
            x: area.x + i * col_width,
            y: area.y,
            width: if i + 1 == count {
                area.width.saturating_sub(col_width * (count - 1))
            } else {
                col_width
            },
            height: area.height,
        })
        .collect()
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

        let cols = column_rects(inner, 7);
        for (row, week) in self.grid.weeks.iter().enumerate() {
            let y = inner.y + row as u16;
            if y >= inner.y + inner.height {
                break;
            }
            for (col, cell) in week.iter().enumerate() {
                let cell_area = cols.get(col).copied().unwrap_or(inner);
                let cell_area = Rect {
                    x: cell_area.x,
                    y,
                    width: cell_area.width,
                    height: 1,
                };
                if cell_area.width == 0 {
                    continue;
                }
                let width = cell_area.width.min(3) as usize;
                let line = day_line(cell, self.view, self.colors, width);
                Paragraph::new(line)
                    .alignment(Alignment::Right)
                    .render(cell_area, buf);
            }
        }
    }
}

#[cfg(test)]
mod tests;
