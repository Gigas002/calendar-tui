use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::calendar::{DayCell, MAX_WEEK_ROWS, MonthGrid};
use crate::render::{
    MINI_WEEK_COL_WIDTH, block_style, column_rects, render_week_number, split_week_column,
    style_for_cell,
};
use crate::settings::Settings;
use crate::theme::ThemeColors;
use crate::view::{ViewMode, ViewState};

/// Rows one mini month takes with its border: a line above and below its six weeks.
const MINI_ROWS_BORDERED: u16 = MAX_WEEK_ROWS as u16 + 2;
/// Rows one mini month takes without a border: a title line and its six weeks.
const MINI_ROWS_BARE: u16 = MAX_WEEK_ROWS as u16 + 1;

/// How the year view fits the terminal's height. The full layout needs 33 lines; below that,
/// first the boxed status and header shrink to one line, then the months lose their borders,
/// so a standard 24-line terminal still shows every day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum YearLayout {
    /// Margin, boxed status and header, bordered months, help line.
    Full,
    /// One status line, bordered months, help line.
    Compact,
    /// One status line, borderless months, help line.
    Bare,
    /// Not even that fits: a message instead.
    TooSmall,
}

impl YearLayout {
    pub(crate) fn for_height(height: u16) -> Self {
        // margin + status box + header box + help line + margin
        if height >= 1 + 3 + 3 + 1 + 1 + 3 * MINI_ROWS_BORDERED {
            Self::Full
        } else if height >= 1 + 1 + 3 * MINI_ROWS_BORDERED {
            Self::Compact
        } else if height >= 1 + 1 + 3 * MINI_ROWS_BARE {
            Self::Bare
        } else {
            Self::TooSmall
        }
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

    let layout = YearLayout::for_height(area.height);
    let grid_area = match layout {
        YearLayout::Full => {
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
        }
        YearLayout::Compact | YearLayout::Bare => {
            let vertical = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ]);
            let [status_area, grid_area, help_area] = vertical.areas(area);
            draw_year_status_line(frame, status_area, settings);
            super::draw_help(frame, help_area, settings);
            grid_area
        }
        YearLayout::TooSmall => {
            draw_too_small(frame, area, settings);
            return;
        }
    };

    frame.render_widget(
        YearOverview {
            grids: &settings.year_grids,
            view: &settings.view,
            colors: &settings.colors,
            show_week_numbers: settings.show_week_numbers,
            bordered: layout != YearLayout::Bare,
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

/// The status and the header on one unboxed line, for short terminals.
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
    let needed = 1 + 1 + 3 * MINI_ROWS_BARE;
    let text = format!(
        "The year view needs {needed} lines (this terminal has {}). m: month view, q: quit",
        area.height
    );
    let widget = Paragraph::new(text)
        .style(settings.colors.cell_base())
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
    let [middle] = Layout::vertical([Constraint::Length(3)])
        .flex(ratatui::layout::Flex::Center)
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
    show_week_numbers: bool,
    bordered: bool,
}

impl Widget for YearOverview<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.width == 0 || area.height == 0 || self.grids.len() < 12 {
            return;
        }

        let rows = Layout::vertical([Constraint::Ratio(1, 1); 3]).split(area);
        for (row_idx, row_area) in rows.iter().enumerate() {
            let cols = Layout::horizontal([Constraint::Ratio(1, 1); 4]).split(*row_area);
            for (col_idx, month_area) in cols.iter().enumerate() {
                let month = (row_idx * 4 + col_idx + 1) as i8;
                let grid = &self.grids[(month - 1) as usize];
                let focused = self.view.focused_month == month;
                MiniMonth {
                    month,
                    grid,
                    view: self.view,
                    colors: self.colors,
                    focused,
                    show_week_numbers: self.show_week_numbers,
                    bordered: self.bordered,
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
    show_week_numbers: bool,
    bordered: bool,
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

        let inner = if self.bordered {
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

        let min_day_width = if self.show_week_numbers {
            7 + MINI_WEEK_COL_WIDTH
        } else {
            7
        };
        if inner.width < min_day_width || inner.height < MAX_WEEK_ROWS as u16 {
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

        // Equal-height week rows; leftover pixels stay below the grid (not between weeks).
        let slots = MAX_WEEK_ROWS as u16;
        let row_height = inner.height / slots;
        if row_height == 0 {
            return;
        }

        let columns = split_week_column(inner, self.show_week_numbers, MINI_WEEK_COL_WIDTH);
        let week_numbers = self.grid.iso_week_numbers();

        for (row_idx, week) in self.grid.weeks.iter().enumerate() {
            let row_y = inner.y.saturating_add(row_idx as u16 * row_height);
            let row_area = Rect {
                x: inner.x,
                y: row_y,
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

            let cols = column_rects(columns.days, 7);
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
