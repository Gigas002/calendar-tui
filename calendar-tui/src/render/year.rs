use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::calendar::{DayCell, MAX_WEEK_ROWS, MonthGrid};
use crate::render::{
    MINI_WEEK_COL_WIDTH, block_style, column_rects, render_week_number, split_week_column,
    style_for_cell,
};
use crate::settings::Settings;
use crate::theme::ThemeColors;
use crate::view::{ViewMode, ViewState};

/// Columns a week of days needs to stay readable: two digits and a space per day.
const DAYS_WIDTH: u16 = 7 * 3;

/// Lines above and below the months: margin, boxed status and header, help line, margin.
const FULL_CHROME_HEIGHT: u16 = 1 + 3 + 3 + 1 + 1;
/// Lines above and below the months when space is short: one status line and the help line.
const COMPACT_CHROME_HEIGHT: u16 = 1 + 1;

/// How the year view fits the terminal: how many months per row, how much decoration, and
/// whether there's room for week numbers. [`YearLayout::choose`] tries the roomiest first and
/// steps down, so the days stay readable (three columns each) at every size it accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct YearLayout {
    /// 4, 3 or 2: twelve months in 3, 4 or 6 rows.
    pub per_row: u16,
    /// Boxed status and header with a margin (otherwise one status line).
    pub full_chrome: bool,
    /// Months in boxes (otherwise a title line and a column of space each side).
    pub bordered: bool,
    pub week_numbers: bool,
}

impl YearLayout {
    /// The roomiest layout that fits `width` x `height`, or `None` when even the tightest
    /// doesn't. Week numbers are dropped only if no layout fits with them.
    pub(crate) fn choose(width: u16, height: u16, week_numbers: bool) -> Option<Self> {
        let week_options: &[bool] = if week_numbers {
            &[true, false]
        } else {
            &[false]
        };
        for &week_numbers in week_options {
            for per_row in [4, 3, 2] {
                for (full_chrome, bordered) in [(true, true), (false, true), (false, false)] {
                    let layout = Self {
                        per_row,
                        full_chrome,
                        bordered,
                        week_numbers,
                    };
                    if layout.width() <= width && layout.height() <= height {
                        return Some(layout);
                    }
                }
            }
        }
        None
    }

    fn rows(self) -> u16 {
        12 / self.per_row
    }

    /// Width one month needs: its days, the week-number column, and a border or padding.
    fn month_width(self) -> u16 {
        let week = if self.week_numbers {
            MINI_WEEK_COL_WIDTH
        } else {
            0
        };
        DAYS_WIDTH + week + 2
    }

    fn month_height(self) -> u16 {
        // Six week rows, plus a border above and below, or a title line.
        MAX_WEEK_ROWS as u16 + if self.bordered { 2 } else { 1 }
    }

    fn width(self) -> u16 {
        let margin = if self.full_chrome { 2 } else { 0 };
        self.per_row * self.month_width() + margin
    }

    fn height(self) -> u16 {
        let chrome = if self.full_chrome {
            FULL_CHROME_HEIGHT
        } else {
            COMPACT_CHROME_HEIGHT
        };
        chrome + self.rows() * self.month_height()
    }
}

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn draw_year(frame: &mut ratatui::Frame, settings: &Settings) {
    let area = frame.area();
    frame.render_widget(ratatui::widgets::Clear, area);
    if !settings.colors.background.is_transparent() {
        frame.render_widget(
            Paragraph::new("").style(settings.colors.background.patch_bg(Style::default())),
            area,
        );
    }

    let Some(layout) = YearLayout::choose(area.width, area.height, settings.show_week_numbers)
    else {
        draw_too_small(frame, area, settings);
        return;
    };

    let grid_area = if layout.full_chrome {
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .margin(1);
        let [status_area, header_area, grid_area, help_area] = vertical.areas(area);
        draw_year_status(frame, status_area, settings);
        draw_year_header(frame, header_area, settings);
        super::draw_help(frame, help_area, settings);
        grid_area
    } else {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [status_area, grid_area, help_area] = vertical.areas(area);
        draw_year_status_line(frame, status_area, settings);
        super::draw_help(frame, help_area, settings);
        grid_area
    };

    frame.render_widget(
        YearOverview {
            grids: &settings.year_grids,
            view: &settings.view,
            colors: &settings.colors,
            layout,
        },
        grid_area,
    );
}

fn status_text(settings: &Settings) -> String {
    let today_line = settings
        .view
        .today
        .strftime(&settings.date_format)
        .to_string();
    format!("{}    Year: {}", today_line, settings.view.view_year)
}

/// The status and the header on one unboxed line, for small terminals.
fn draw_year_status_line(frame: &mut ratatui::Frame, area: Rect, settings: &Settings) {
    let focused = MONTH_NAMES[(settings.view.focused_month - 1) as usize];
    let widget = Paragraph::new(format!(" {} — {}", status_text(settings), focused))
        .style(
            settings
                .colors
                .status
                .patch_fg(settings.colors.background.patch_bg(Style::default())),
        )
        .alignment(Alignment::Center);
    frame.render_widget(widget, area);
}

fn draw_too_small(frame: &mut ratatui::Frame, area: Rect, settings: &Settings) {
    let tightest = YearLayout {
        per_row: 4,
        full_chrome: false,
        bordered: false,
        week_numbers: false,
    };
    let text = format!(
        "The year view needs at least {}x{} (this terminal is {}x{}). m: month view, q: quit",
        tightest.width(),
        tightest.height(),
        area.width,
        area.height
    );
    let widget = Paragraph::new(text)
        .style(settings.colors.cell_base())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    let [middle] = Layout::vertical([Constraint::Length(3)])
        .flex(Flex::Center)
        .areas(area);
    frame.render_widget(widget, middle);
}

fn draw_year_status(frame: &mut ratatui::Frame, area: Rect, settings: &Settings) {
    let text = status_text(settings);

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
fn draw_year_header(frame: &mut ratatui::Frame, area: Rect, settings: &Settings) {
    let focused = MONTH_NAMES[(settings.view.focused_month - 1) as usize];
    let text = format!("{} — {}", settings.view.view_year, focused);

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

struct YearOverview<'a> {
    grids: &'a [MonthGrid],
    view: &'a ViewState,
    colors: &'a ThemeColors,
    layout: YearLayout,
}

impl Widget for YearOverview<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.width == 0 || area.height == 0 || self.grids.len() < 12 {
            return;
        }

        let per_row = self.layout.per_row as usize;
        let row_count = 12 / per_row;
        let rows =
            Layout::vertical(vec![Constraint::Ratio(1, row_count as u32); row_count]).split(area);
        for (row_idx, row_area) in rows.iter().enumerate() {
            let cols = Layout::horizontal(vec![Constraint::Ratio(1, per_row as u32); per_row])
                .split(*row_area);
            for (col_idx, month_area) in cols.iter().enumerate() {
                let month = (row_idx * per_row + col_idx + 1) as i8;
                let grid = &self.grids[(month - 1) as usize];
                MiniMonth {
                    month,
                    grid,
                    view: self.view,
                    colors: self.colors,
                    focused: self.view.focused_month == month,
                    layout: self.layout,
                }
                .render(*month_area, buf);
            }
        }
    }
}

struct MiniMonth<'a> {
    month: i8,
    grid: &'a MonthGrid,
    view: &'a ViewState,
    colors: &'a ThemeColors,
    focused: bool,
    layout: YearLayout,
}

impl Widget for MiniMonth<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.width < 4 || area.height < 4 {
            return;
        }

        let title = MONTH_NAMES[(self.month - 1) as usize];
        let border_style = if self.focused {
            self.colors
                .selected
                .patch_fg(Style::default())
                .add_modifier(Modifier::BOLD)
        } else {
            self.colors.border.patch_fg(Style::default())
        };

        let inner = if self.layout.bordered {
            let block = Block::default()
                .title(format!(" {title} "))
                .borders(Borders::ALL)
                .border_style(border_style)
                .style(block_style(self.colors));
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            // No border to color: the focused month's title is reversed instead. A column
            // on each side keeps neighbouring months' days apart.
            let title_style = if self.focused {
                border_style.add_modifier(Modifier::REVERSED)
            } else {
                border_style
            };
            Paragraph::new(format!(" {title} "))
                .style(title_style)
                .alignment(Alignment::Center)
                .render(Rect { height: 1, ..area }, buf);
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height - 1,
            }
        };

        if inner.height < MAX_WEEK_ROWS as u16 {
            return;
        }

        let year_view = ViewState {
            mode: ViewMode::Year,
            view_year: self.view.view_year,
            view_month: self.month,
            focused_month: self.month,
            selected_day: None,
            selection_mode: false,
            today: self.view.today,
        };

        // Equal-height week rows; leftover lines stay below the grid (not between weeks).
        let row_height = inner.height / MAX_WEEK_ROWS as u16;
        let columns = split_week_column(inner, self.layout.week_numbers, MINI_WEEK_COL_WIDTH);
        let week_numbers = self.grid.iso_week_numbers();
        let cols = column_rects(columns.days, 7);

        for (row_idx, week) in self.grid.weeks.iter().enumerate() {
            let row_area = Rect {
                x: inner.x,
                y: inner.y.saturating_add(row_idx as u16 * row_height),
                width: inner.width,
                height: row_height,
            };

            if let Some(week_col) = columns.week_col {
                let week_area = Rect {
                    x: week_col.x,
                    y: row_area.y,
                    width: week_col.width,
                    height: row_area.height,
                };
                if let Some(&iso_week) = week_numbers.get(row_idx) {
                    render_week_number(iso_week, self.colors, week_area, buf);
                }
            }

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
                render_mini_day(cell, &year_view, self.colors, cell_area, buf);
            }
        }
    }
}

fn render_mini_day(
    cell: &DayCell,
    view: &ViewState,
    colors: &ThemeColors,
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
) {
    let day_str = cell.day.to_string();

    let style = if cell.date == view.today {
        colors
            .today
            .patch_fg(colors.background.patch_bg(Style::default()))
            .add_modifier(Modifier::BOLD)
    } else {
        style_for_cell(cell, view, colors)
    };

    let text_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1.min(area.height),
    };
    Paragraph::new(Line::from(Span::styled(day_str, style)))
        .alignment(Alignment::Center)
        .render(text_area, buf);
}
