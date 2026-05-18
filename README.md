# calendar-tui

A minimal, fast **terminal calendar** built with [ratatui](https://crates.io/crates/ratatui). No network, no sync — just month and year views, keyboard navigation, and TOML config/themes.

## Preview

**Month view** (press `m` from year view, or set `default_mode = "month"`):

```text
┌ Status ──────────────────────────────────────┐
│  Mon, 18 May 2026                   Weeks: 6   │
├ May 2026 ────────────────────────────────────┤
│           May 2026                           │
├ Wk  Mo  Tu  We  Th  Fr  Sa  Su ────────────────┤
│                 1   2   3   4   5   6   7    │
│   8   9  10  11  12  13  14  15  16  17  18  │
│  ...                                         │
└ h/l month  j/k year  esc year-view  q quit ──┘
```

**Year view** (default): twelve mini month grids for the focused year; `Enter` opens the focused month.

Colors come from your theme (example uses a Catppuccin-inspired palette). Today and weekends are highlighted; adjacent-month days use `other_month`.

## Install

Requires a **Rust** toolchain ([rustup](https://rustup.rs/)).

```bash
# From a clone of this repository
cargo install --path calendar-tui

# Or build a release binary locally
cargo build --release -p calendar-tui
# binary: target/release/calendar-tui
```

Run only in a real terminal (not piped stdout):

```bash
calendar-tui
```

## Keybindings

Global:

| Key | Action |
| --- | --- |
| `q` | Quit |
| `Ctrl+c` | Quit |
| `m` | Month view |
| `y` | Year view |
| `Esc` | Back to year view (from month view) |

**Month view** (navigation):

| Key | Action |
| --- | --- |
| `Left` / `h` | Previous month |
| `Right` / `l` | Next month |
| `Up` / `k` | Previous year |
| `Down` / `j` | Next year |
| `PgUp` / `PgDn` | Previous / next month |
| `Home` / `t` | Jump view to today |
| `Enter` / `Space` | Toggle day selection mode |

**Month view** (selection mode, after `Space`):

| Key | Action |
| --- | --- |
| `Left` / `Right` / `h` / `l` | Previous / next day |
| `Up` / `Down` / `k` / `j` / `PgUp` / `PgDn` | Previous / next week |
| `Space` / `Enter` | Exit selection mode |
| `Home` / `t` | Jump to today |

**Year view**:

| Key | Action |
| --- | --- |
| `Left` / `h` | Focus previous month |
| `Right` / `l` | Focus next month |
| `Up` / `k` | Previous year |
| `Down` / `j` | Next year |
| `PgUp` / `PgDn` | Previous / next year |
| `Home` / `t` | Jump to today |
| `Enter` | Open focused month (month view) |

A help line at the bottom of the screen summarizes bindings for the current mode.

## Configuration

### Paths

| File | Default location |
| --- | --- |
| Config | `$XDG_CONFIG_HOME/calendar-tui/config.toml` (falls back to `~/.config/calendar-tui/config.toml`) |
| Theme | Resolved from `[calendar].theme` in config (see below) |

Override paths on the CLI:

```bash
calendar-tui --config /path/to/config.toml --theme /path/to/theme.toml
```

**Theme resolution** (first match wins):

1. `--theme` if given
2. Same directory as the config file: `theme.toml`
3. `themes/theme.toml` next to the config file
4. `$XDG_CONFIG_HOME/calendar-tui/themes/<name>`
5. Default path under XDG themes dir (file may be missing — built-in defaults apply)

Copy the examples to get started:

```bash
mkdir -p ~/.config/calendar-tui/themes
cp examples/config.toml ~/.config/calendar-tui/config.toml
cp examples/theme.toml ~/.config/calendar-tui/themes/theme.toml
```

Missing config or theme files use **built-in defaults** (Monday week start, year view, default palette).

### `config.toml`

See [`examples/config.toml`](examples/config.toml).

| Section / key | Description |
| --- | --- |
| `[calendar].week_start` | First column of the grid: `monday` … `sunday` |
| `[calendar].show_week_numbers` | Show an ISO week-number column to the left of the grid (default `false`) |
| `[calendar].theme` | Theme file name or path (default `theme.toml`) |
| `[display].default_mode` | Starting view: `month` or `year` |
| `[display].date_format` | strftime-style format for today on the status line |
| `[display].month_year_format` | Format for the month/year header |

### `theme.toml`

See [`examples/theme.toml`](examples/theme.toml).

Colors are `#RRGGBB` or `#RRGGBBAA`. Use alpha `00` on `background` for a transparent background (terminal default shows through).

| Section / key | Used for |
| --- | --- |
| `[base].background` | Screen background |
| `[base].foreground` | Default text |
| `[base].border` | Widget borders |
| `[calendar].header` | Month/year titles |
| `[calendar].status` | Status line |
| `[calendar].today` | Today’s cell |
| `[calendar].selected` | Selected day |
| `[calendar].weekend` | Saturday and Sunday |
| `[calendar].other_month` | Leading/trailing days outside the view month |

## Development

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

See [`docs/PLAN.md`](docs/PLAN.md) for architecture and phased roadmap.

## License

GPL-3.0-only — see [LICENSE](LICENSE).
