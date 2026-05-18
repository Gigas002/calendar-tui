use std::io::{self, IsTerminal, Stdout, stdout};
use std::time::Duration;

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, Terminal};

use crate::date::naive_from_ymd;
use crate::error::Error;
use crate::settings::Settings;

const HELP: &str = "Press q to quit";

struct App {
    running: bool,
    settings: Settings,
}

impl App {
    fn new(settings: Settings) -> Self {
        Self {
            running: true,
            settings,
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}

struct Tty {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tty {
    fn enter() -> Result<Self, Error> {
        enable_raw_mode().map_err(Error::Terminal)?;
        stdout()
            .execute(EnterAlternateScreen)
            .map_err(Error::Terminal)?;
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).map_err(Error::Terminal)?;
        Ok(Self { terminal })
    }

    fn leave(&mut self) -> Result<(), Error> {
        disable_raw_mode().map_err(Error::Terminal)?;
        stdout()
            .execute(LeaveAlternateScreen)
            .map_err(Error::Terminal)?;
        self.terminal.show_cursor().map_err(Error::Terminal)?;
        Ok(())
    }
}

impl Drop for Tty {
    fn drop(&mut self) {
        let _ = self.leave();
    }
}

pub fn run(settings: Settings) -> Result<(), Error> {
    if !io::stdout().is_terminal() {
        return Err(Error::NotATty);
    }

    let mut app = App::new(settings);
    let mut tty = Tty::enter()?;

    while app.running {
        tty.terminal
            .draw(|frame| draw(frame, &app.settings))
            .map_err(Error::Terminal)?;

        if event::poll(Duration::from_millis(100)).map_err(Error::Terminal)? {
            let Event::Key(key) = event::read().map_err(Error::Terminal)? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                _ => {}
            }
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame<'_>, settings: &Settings) {
    let colors = &settings.colors;
    let area = frame.area();
    let vertical = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .margin(1);

    let [status_area, header_area, body_area, help_area] = vertical.areas(area);

    let today_line = settings
        .view
        .today
        .format(&settings.date_format)
        .to_string();
    let status_text = format!("{}    Weeks: {}", today_line, settings.grid.week_count());

    let header_text = naive_from_ymd(settings.view.view_year, settings.view.view_month, 1)
        .map(|d| d.format(&settings.month_year_format).to_string())
        .unwrap_or_else(|| format!("{} / {}", settings.view.view_month, settings.view.view_year));

    let body = format!(
        "Phase 1 — grid {}×{} (week_start: {:?})",
        settings.grid.week_count(),
        7,
        settings.week_start
    );

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(colors.status))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Today "),
        );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(colors.header))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border)),
        );

    let body_widget = Paragraph::new(body)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border)),
        )
        .alignment(Alignment::Center);

    let help = Paragraph::new(HELP).alignment(Alignment::Center).style(
        Style::default()
            .fg(colors.foreground)
            .add_modifier(Modifier::DIM),
    );

    frame.render_widget(status, status_area);
    frame.render_widget(header, header_area);
    frame.render_widget(body_widget, body_area);
    frame.render_widget(help, help_area);
}
