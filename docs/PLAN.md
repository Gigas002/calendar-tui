# calendar-tui — Rust architecture + implementation plan

This document is the **human roadmap** and **agent playbook** for **calendar-tui**: a minimal, **interactive terminal calendar** built with **[ratatui](https://crates.io/crates/ratatui)** and **crossterm** (or the ratatui-recommended terminal stack). It should feel **simple and lightning fast** — no network, no heavy UI layers, no GUI toolkits.

It mirrors the **execution discipline** of `docs/ABAR_PLAN.md` (and `wau/docs/WAU_RS_PLAN.md`), adapted for a **single binary crate**:

- One crate (`calendar-tui`), clear internal modules, small verifiable phases, strict quality gates (fmt, clippy `-D warnings` with feature matrix, tests, `cargo doc`, `typos`, `cargo deny`).
- **Directory modules** with **sibling `tests.rs`** — tests never live in the same file as logic.
- **Cargo features** only when optional code paths would otherwise bitrot CI (keep defaults lean for v0.1.0).

**Reference configs (source of truth for schemas, to be added with Phase 1):**

- `examples/config.toml` — week start, locale/format hints, default view, key bindings (if not hardcoded initially).
- `examples/theme.toml` — colors for today, headers, grid, borders, notes (notes styling reserved for post-0.1.0).

---

## 1. Goals and constraints

### 1.1 Goals

- **Minimal surface area**: one primary view (month grid + status/header); every extra capability behind clear phases or **post-0.1.0** scope.
- **Fast and local**: calendar math and redraw only; startup and month navigation should be imperceptible on a typical terminal.
- **Always show context**:
  - **Current date** (today) — visually distinct from other days.
  - **Current month** and **current year** in the focused view (header or status line).
  - **Week count** for the displayed month (see §1.4).
- **Navigation**: scroll **month-by-month** and **year-by-year** (keyboard; mouse optional later if cheap).
- **Week layout**: configurable **first day of week** (Monday, Sunday, Saturday, etc.) via config.
- **Themes**: TOML color palette; hot-reload **not** required for v0.1.0 — load at startup + optional reload key is enough.
- **Config discovery**: XDG-style paths (e.g. `$XDG_CONFIG_HOME/calendar-tui/config.toml`, themes under `.../calendar-tui/themes/`), plus `--config` / `--theme` on the binary.

### 1.2 Discipline (non-negotiable, from ABAR / WAU)

- **Single crate**: all code lives in **`calendar-tui`**; split by **modules**, not workspace members.
- **Module boundaries**: pure calendar/date/view logic stays free of ratatui draw calls where practical so unit tests need no terminal; `app/` owns the event loop and terminal setup; `config/` / `theme/` own TOML and paths.
- **Synchronous TUI loop**: one thread, `crossterm` poll + ratatui `draw`; no async runtime for v0.1.0. If background work is added later (e.g. file watchers), gate it behind a feature and never block draw on I/O.
- **Step sizing**: small PR-sized phases with explicit **Verify** blocks.
- **Feature matrix in CI**: default, `--all-features`, `--no-default-features` (core must still build: month view + navigation).
- **Naming**: short, descriptive; prefer clarity over abstraction depth.
- **Code comments**: describe current behavior only (invariants, layout rules, non-obvious date math). No roadmap phase labels in source.
- **No `println!` in library-style modules** — use `tracing` (subscriber wired in `main` only).

### 1.3 Non-goals (v0.1.0)

- **No** separate `libcalendar` (or other) library crate — the app is small enough for one package.
- **No** Google/CalDAV/Exchange sync, reminders, or event CRUD.
- **No** pluggable **date notes / colored markers** in the first release (planned post-0.1.0, §8).
- **No** multi-pane week/agenda/year views unless time remains after v0.1.0 polish — month grid is the product.
- **No** web UI, iced, egui, or GTK.

### 1.4 Definitions

- **View date**: the `(year, month)` (and optionally **selected day**) the UI is focused on; may differ from **today**.
- **Today**: local calendar date from the system clock (`chrono::Local` or equivalent).
- **Weeks in month**: number of **calendar rows** (weeks) required to display that month in a 7-column grid honoring `week_start` — i.e. count of weeks from the first grid cell for the month through the last day, **not** ISO week-of-year numbering unless explicitly labeled elsewhere in the UI.
- **Week start**: which weekday occupies column 0 (e.g. `monday`, `sunday`); serde string enum in config.
- **Theme**: resolved RGB/hex colors mapped to ratatui `Style` (and optional modifiers for today/selected/weekend).
- **Calendar provider** (post-0.1.0): pluggable source of per-date annotations (color, label, icon) from config files or future backends.

---

## 2. Repository layout (target)

```text
calendar-tui/                  # workspace root
  Cargo.toml                   # workspace members: ["calendar-tui"]
  Cargo.lock                   # committed
  deny.toml
  examples/
    config.toml
    theme.toml
  calendar-tui/
    Cargo.toml                 # ratatui, crossterm, chrono, clap, toml, tracing, …
    src/
      main.rs                  # tracing init, CLI, load settings, run app
      error.rs                 # thiserror (config, theme, terminal, date math)
      app/
        mod.rs                 # terminal setup, loop, dispatch keys
        tests.rs
      cli/
        mod.rs
        tests.rs
      config/
        mod.rs                 # TOML parse, XDG paths, --config
        tests.rs
      settings/
        mod.rs                 # merged: cli > env > config
        tests.rs
      calendar/                # month grid, week rows, day cells
        mod.rs
        tests.rs
      date/                    # today, days-in-month, week-start helpers
        mod.rs
        tests.rs
      view/                    # focused month, selection, navigation
        mod.rs
        tests.rs
      render/                  # ratatui widgets / custom Widget
        mod.rs
        tests.rs
      theme/
        mod.rs                 # load theme.toml, hex → ratatui Color/Style
        tests.rs
      provider/                # (post-0.1.0, optional feature) notes trait + file backend
        mod.rs
        tests.rs
  docs/
    PLAN.md                    # this file
    ABAR_PLAN.md               # structural reference
  .github/workflows/           # fmt-clippy, test, doc, typos, deny, build
```

**Module rules**

- `calendar/`, `date/`, `view/`: no filesystem I/O; no ratatui imports if avoidable (easier unit tests).
- `render/`: ratatui only; takes plain structs (`MonthGrid`, `ViewState`, resolved theme styles).
- `config/`, `theme/`, `settings/`: load and merge user files; `app/` receives a single resolved `Settings` struct.

---

## 3. Data model and config

### 3.1 `config.toml` (see `examples/config.toml`)

**Intent**

- **`[calendar]`** (or top-level keys):
  - `week_start` — `monday` | `sunday` | `saturday` | … (document supported set in example).
  - `show_week_numbers` — optional bool (column of ISO or ordinal week numbers; default `false` for v0.1.0 unless trivial).
- **`[display]`**:
  - `date_format` — strftime-style or preset for header “today” line (default locale-friendly).
  - `month_year_format` — header for focused month/year.
- **`[keys]`** (optional): override default bindings; if omitted, use built-in defaults (§4.2).

**Invariants**

- Invalid `week_start` → deserialize or resolve error at startup with a clear message.
- Missing file → use built-in defaults (Monday week start, system locale date formatting).

### 3.2 `theme.toml` (see `examples/theme.toml`)

**Intent**

- Global: `background`, `foreground`, `border`.
- Calendar-specific: `today`, `selected`, `weekend`, `other_month`, `header`, `status`.
- Post-0.1.0: `note_default`, `note_*` or per-provider color keys (see §8).

Colors as `#RRGGBB` or `#RRGGBBAA` hex strings; `theme` module converts to ratatui `Color`.

### 3.3 Runtime state (`view`)

| Field            | Meaning                                      |
| ---------------- | -------------------------------------------- |
| `view_year`      | Focused year                                 |
| `view_month`     | Focused month (1–12)                         |
| `selected_day`   | Optional highlighted day within view month   |
| `today`          | Cached local today (refresh on draw or tick) |

Navigation mutates `view_year` / `view_month` and clamps `selected_day` when the day does not exist in the new month.

---

## 4. UI and interaction

### 4.1 Layout (ratatui)

Single screen, roughly:

```text
┌─ Status / today ─────────────────────────────┐
│  Mon, 18 May 2026          Weeks: 6          │
├─ Month header ───────────────────────────────┤
│           May 2026                           │
├─ Weekday headers ────────────────────────────┤
│  Mo  Tu  We  Th  Fr  Sa  Su                  │
├─ Month grid (variable rows) ─────────────────┤
│       1   2   3   4   5   6   7              │
│   8   9  ...                                 │
└─ Help line (optional, dim) ──────────────────┘
```

- **Status line**: **current date** (today) + **week count** for the **displayed** month.
- **Header**: **current month** and **current year** of the **view** (not necessarily today’s month).
- **Grid**: 7 columns from `week_start`; leading/trailing days from adjacent months styled with `other_month` if theme provides it.

Prefer a **custom `Widget`** or small set of widgets over deep widget trees to keep measure/draw predictable and fast.

### 4.2 Default key bindings (v0.1.0)

| Key              | Action                    |
| ---------------- | ------------------------- |
| `q` / `Ctrl+c`   | Quit                      |
| `Left` / `h`     | Previous month            |
| `Right` / `l`    | Next month                |
| `Up` / `k`       | Previous year             |
| `Down` / `j`     | Next year                 |
| `Home` / `t`     | Jump view to today        |
| `Enter` / `Space`| Toggle/select day (optional highlight) |

PgUp/PgDn may alias month navigation if not conflicting with terminal defaults.

### 4.3 Rendering performance

- Rebuild **month grid cells** only when `view_year`, `view_month`, or `week_start` changes — not every frame.
- Full terminal redraw per input event is acceptable for v0.1.0; avoid allocating large strings in the hot path (use buffers or preformatted weekday headers).

### 4.4 Damage / refresh

- No partial terminal damage API required; ratatui full draw each event is fine at terminal scale.

---

## 5. Dependencies (policy)

| Crate        | Role                                      |
| ------------ | ----------------------------------------- |
| `ratatui`    | TUI layout and draw                       |
| `crossterm`  | Raw mode, input, terminal size            |
| `chrono`     | Dates, month length, local today          |
| `serde`      | Config/theme deserialization              |
| `clap`       | CLI                                       |
| `thiserror`  | Error types                               |
| `tracing`    | Logging (`tracing-subscriber` in `main`)  |

- **Edition**: `2024` (workspace).
- **Versions**: `x.y` or `x` in manifests; lockfile committed.
- **Health**: prefer maintained crates; avoid pulling a full async stack for v0.1.0.

---

## 6. Quality gates (mirror ABAR §7)

Whenever a phase is marked complete:

- `cargo fmt --check`
- `typos`
- `cargo deny check` (populate `deny.toml` **allow** list as licenses are introduced)
- `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --no-default-features`
- `cargo test --workspace --all-features`
- `cargo doc --workspace --no-deps`

### 6.1 Test discipline

- Unit tests in **`tests.rs`** next to `mod.rs` per directory module.
- **`calendar/`**, **`date/`**, **`view/`**: week-start grids, week-count per month, leap years, navigation clamping — **no terminal**.
- **`config/`**, **`theme/`**: deserialize `examples/*.toml`.
- Optional: ratatui `TestBackend` in **`render/`** (feature-gated if flaky in CI).

### 6.2 CI

Use existing workflows (`build`, `fmt-clippy`, `test`, `doc`, `typos`, `deny`); no Cairo/Pango deps. Matrix: default / all-features / no-default-features.

---

## 7. Phased steps

### Phase 0 — Workspace + hygiene + empty vertical slice

- [x] Ensure root `Cargo.toml` workspace member is **`calendar-tui`** only; flesh out `calendar-tui/Cargo.toml` deps.
- [x] `main` + `app`: tracing subscriber, minimal TUI (alternate screen, static title, quit on `q`); structured error on missing terminal.
- [x] Populate **`deny.toml` licenses allow list** for initial deps.

**Verify**: all gates in §6; manual run in a real terminal.

### Phase 1 — Config + theme + date model

- [ ] Serde models for `examples/config.toml` / `theme.toml` (`week_start`, colors).
- [ ] XDG path resolution + `--config` / `--theme` (`cli` + `config` + `theme`).
- [ ] `date`: today, days-in-month, weekday index with `week_start`.
- [ ] `calendar`: build month grid (`Vec` of weeks, 7 cells each with day + in-month flag).

**Verify**: unit tests for grid shape and week count (Feb leap/non-leap, each weekday as month start).

### Phase 2 — Render month view

- [ ] `render`: status (today), header (view month/year), weekday row, grid with today highlighted.
- [ ] Wire theme colors into `Style`; defaults when keys missing.
- [ ] Display **week count** on status line.

**Verify**: `TestBackend` or manual check; today highlight when view month ≠ today’s month.

### Phase 3 — Navigation

- [ ] `view`: prev/next month, prev/next year, jump to today, optional day selection.
- [ ] `app`: map keys (§4.2), redraw on change.
- [ ] Clamp selection on shorter months (e.g. Jan 31 → Feb).

**Verify**: unit tests for state transitions; manual scroll across years.

### Phase 4 — Week start + theme polish

- [ ] Apply `week_start` from config to grid and weekday headers.
- [ ] Complete `theme.toml` coverage (weekend, other_month, border, status).
- [ ] `settings`: optional CLI overrides (e.g. `--week-start`).

**Verify**: `monday` vs `sunday` grid tests; theme parse tests.

### Phase 5 — Polish + v0.1.0 release

- [ ] README: install, keybindings, config/theme paths, screenshots/gif.
- [ ] `examples/*.toml` documented and validated in tests.
- [ ] CHANGELOG; tag **v0.1.0**.

**Verify**: full §6 gates + manual dogfood for a full year of navigation.

### Post-0.1.0 — Pluggable calendar notes

- [ ] Optional feature **`provider`**: trait `CalendarProvider` → per-date annotations (color, short label).
- [ ] File backend: TOML/JSON (e.g. `~/.config/calendar-tui/notes.toml`); optional `r` reload key.
- [ ] `render`: note indicator on cells using theme `note_*` colors.
- [ ] Document extension point for future backends without implementing them.

**Verify**: fixture file tests; manual run with sample notes.

---

## 8. Definition of done (v0.1.0)

- [ ] Terminal app starts and quits cleanly (`q` / `Ctrl+c`).
- [ ] Shows **today’s date**, **focused month**, **focused year**, and **week count** for the displayed month.
- [ ] Month grid correct for all months with configurable **week start** (at least Monday and Sunday).
- [ ] Navigate **months** and **years** via keyboard without perceptible lag.
- [ ] **Colors/themes** from `theme.toml` with documented defaults.
- [ ] Config from XDG path or `--config`; theme from themes dir or `--theme`.
- [ ] **No** pluggable date notes required for this milestone.
- [ ] CI green on default / all-features / no-default-features; docs build.

---

## 9. Document maintenance

Update this plan when:

- v0.1.0 scope changes (especially notes moving in or out of the first release)
- `examples/*.toml` schema changes — update examples first, then this doc
- new optional features add Cargo feature flags
- §1.2 **Code comments** rule changes

---

## Revision history

| Date       | Change                                                                 |
| ---------- | ---------------------------------------------------------------------- |
| 2026-05-18 | Initial calendar-tui plan from ABAR_PLAN discipline + product requirements |
| 2026-05-18 | Drop `libcalendar`; single `calendar-tui` crate with internal modules   |
