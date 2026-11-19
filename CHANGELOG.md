# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Dates use `jiff` instead of `chrono`, which is unmaintained (RUSTSEC-2026-0543). Navigation
  now stops at the years -9998 and 9998.
- Logging: only warnings and errors by default (it was `info`, so every start left a line in
  the shell), without timestamps, and without color codes when stderr isn't a terminal.
  `RUST_LOG` takes a level (`RUST_LOG=debug`); per-module filters are gone.

### Fixed

- The year view needed 33 lines: on a 24-line terminal every month was an empty box, and up to
  32 lines the last row was. Below 33 lines the status and header now share one line, below
  26 the months lose their borders, and below 23 a message points to the month view.
- Left open past midnight, the calendar kept showing yesterday as today. It now moves to the
  new day at midnight.
- The calendar redrew itself ten times a second while idle. It now redraws only on a key, a
  resize or a new day, and otherwise doesn't wake up at all (until midnight, or once a
  minute at most).
- In month view, selecting today (what `Space` does first) showed no selection: today's style
  hid it. Today is now shown reversed when selected.
- A config or theme file that doesn't exist was silently replaced by the defaults, even when
  named with `--config`, `--theme` or the config's `theme` key. Named files must now exist. A
  missing file at the default location still means the defaults, and no longer prints a
  warning on every start.
- Unknown sections and keys in `config.toml` and the theme were ignored, so a typo like
  `weekstart = "sunday"` silently kept the default. They're now errors that name the key.
- A theme color with a non-ASCII character (`"#aéééb"`) crashed `calendar-tui`, and one with a
  sign (`"#+f+f+f"`) was accepted. Both are now errors, and color errors name the key, the value
  and the theme file.
- An unknown directive in `date_format` or `month_year_format` (a typo like `%Q`) crashed
  `calendar-tui` on its first draw. It's now shown as written.
- A crash while the calendar was on screen printed its message to the alternate screen, which
  was then cleared: `calendar-tui` exited with status 101 and no explanation. The terminal is
  now restored first, so the message stays visible. If setting up the terminal fails half-way,
  raw mode is turned off again.

## [0.1.0] - 2026-05-18

### Added

- Interactive terminal calendar with **month** and **year** views (ratatui + crossterm).
- Status line: local **today**, **week count** for the displayed month.
- Month grid with configurable **week start** (`monday` … `sunday`), today/weekend/other-month styling.
- Keyboard navigation: months, years, jump to today, optional day selection in month view.
- TOML **config** and **theme** with XDG paths and `--config` / `--theme` overrides.
- Example files under `examples/` validated in unit tests.
- Optional **ISO week-number** column (`show_week_numbers`) in month and year views.

### Notes

- No event sync, reminders, or per-date notes (planned post-0.1.0).

[0.1.0]: https://github.com/Gigas002/calendar-tui/releases/tag/v0.1.0
