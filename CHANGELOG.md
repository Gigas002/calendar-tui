# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
