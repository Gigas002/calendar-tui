use std::io::{self, IsTerminal, Stdout, stdout};
use std::time::Duration;

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.quit(),
            _ if self.settings.view.selection_mode => self.handle_selection_key(key),
            _ => self.handle_navigate_key(key),
        }
    }

    fn handle_navigate_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.settings.view.prev_month();
                self.settings.sync_view();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.settings.view.next_month();
                self.settings.sync_view();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.settings.view.prev_year();
                self.settings.sync_view();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.settings.view.next_year();
                self.settings.sync_view();
            }
            KeyCode::Home | KeyCode::Char('t') => {
                self.settings.view.jump_to_today();
                self.settings.sync_view();
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.settings.view.toggle_selection_mode();
            }
            KeyCode::PageUp => {
                self.settings.view.prev_month();
                self.settings.sync_view();
            }
            KeyCode::PageDown => {
                self.settings.view.next_month();
                self.settings.sync_view();
            }
            _ => {}
        }
    }

    fn handle_selection_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.settings.view.toggle_selection_mode();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.settings.view.select_prev_day();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.settings.view.select_next_day();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.settings.view.select_prev_week();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.settings.view.select_next_week();
            }
            KeyCode::PageUp => {
                self.settings.view.select_prev_week();
            }
            KeyCode::PageDown => {
                self.settings.view.select_next_week();
            }
            KeyCode::Home | KeyCode::Char('t') => {
                self.settings.view.jump_to_today();
                self.settings.sync_view();
            }
            _ => {}
        }
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
            app.handle_key(key);
        }
    }

    Ok(())
}
