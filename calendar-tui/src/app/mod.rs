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
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, Terminal};

use crate::error::Error;

const TITLE: &str = "calendar-tui";
const HELP: &str = "Press q to quit";

struct App {
    running: bool,
}

impl App {
    fn new() -> Self {
        Self { running: true }
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

pub fn run() -> Result<(), Error> {
    if !io::stdout().is_terminal() {
        return Err(Error::NotATty);
    }

    let mut app = App::new();
    let mut tty = Tty::enter()?;

    while app.running {
        tty.terminal.draw(draw).map_err(Error::Terminal)?;

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

fn draw(frame: &mut Frame<'_>) {
    let area = frame.area();
    let vertical = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .margin(1);

    let [title_area, body_area, help_area] = vertical.areas(area);

    let title = Paragraph::new(Line::from(Span::styled(
        TITLE,
        Style::default().add_modifier(Modifier::BOLD),
    )))
    .block(Block::default().borders(Borders::ALL).title(" Welcome "))
    .alignment(Alignment::Center);

    let body = Paragraph::new("Phase 0 — month view coming soon.")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    let help = Paragraph::new(HELP)
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::DIM));

    frame.render_widget(title, title_area);
    frame.render_widget(body, body_area);
    frame.render_widget(help, help_area);
}
