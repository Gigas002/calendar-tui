use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::calendar::{DayCell, MonthGrid, MAX_WEEK_ROWS};
use crate::render::{block_style, column_rects, style_for_cell};
use crate::settings::Settings;
use crate::theme::ThemeColors;
use crate::view::{ViewMode, ViewState};

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
    frame.render_widget(
        YearOverview {
            grids: &settings.year_grids,
            view: &settings.view,
            colors: &settings.colors,
        },
        grid_area,
    );
    super::draw_help(frame, help_area, settings);
}

fn draw_year_status(frame: &mut ratatui::Frame, area: Rect, settings: &Settings) {
    let today_line = settings
        .view
        .today
        .format(&settings.date_format)
        .to_string();
    let text = format!("{}    Year: {}", today_line, settings.view.view_year);

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
                let month = (row_idx * 4 + col_idx + 1) as u32;
                let grid = &self.grids[(month - 1) as usize];
                let focused = self.view.focused_month == month;
                MiniMonth {
                    month,
                    grid,
                    view: self.view,
                    colors: self.colors,
                    focused,
                }
                .render(*month_area, buf);
            }
        }
    }
}

struct MiniMonth<'a> {
    month: u32,
    grid: &'a MonthGrid,
    view: &'a ViewState,
    colors: &'a ThemeColors,
    focused: bool,
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

        let block = Block::default()
            .title(format!(" {title} "))
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(block_style(self.colors));
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 7 || inner.height < MAX_WEEK_ROWS as u16 {
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

        for (row_idx, week) in self.grid.weeks.iter().enumerate() {
            let row_y = inner.y.saturating_add(row_idx as u16 * row_height);
            let row_area = Rect {
                x: inner.x,
                y: row_y,
                width: inner.width,
                height: row_height,
            };
            let cols = column_rects(row_area, 7);
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
