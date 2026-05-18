use std::io::{self, IsTerminal, Stdout, stdout};
use std::time::Duration;

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::error::Error;
use crate::render;
use crate::settings::Settings;

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
            .draw(|frame| render::draw(frame, &app.settings))
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
