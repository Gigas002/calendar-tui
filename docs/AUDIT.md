# Audit

A whole-application audit of calendar-tui (2026-11-19, from `1f446b3`) in five parts: bugs
(§2), performance (§3), security and robustness (§4), usability (§5), and code quality, tests
and project hygiene (§6). §7 lists what was left open, and §8 is the checklist to re-run the
audit.

Each finding says what was found, how serious it is, and what was done: **fixed** (with the
commit), **accepted** (with the reason), or **OK** (checked, nothing to do). Every fix is its
own commit, with a regression test that fails without it where one can be written.

Severity: **high** (a crash or an unusable screen in normal use, or a failure with no
explanation), **medium** (wrong behavior in a reachable case, an ignored setting, a large
waste), **low** (hardening, polish, or needs an unusual setup).

## 1. Method

Every source file, test, example, workflow and manifest was read. Findings were reproduced
three ways:

- **Probe tests** in the crate's own harness: ratatui's `TestBackend` for screens (every size
  from 0x0 to 39x29 in both views, selection on and off), and config and theme parsing.
- **The release binary** in a private tmux server (its own socket), with `HOME` set to a
  scratch directory and nothing else in the environment: screen captures at several sizes,
  key sequences, a panic triggered from a temporary build (not committed), and midnight
  reached by setting `TZ` to a fixed offset whose midnight was 20 seconds away.
- **Measurements** of the base (`1f446b3`) and final release builds: CPU time and wakeups
  from `/proc/<pid>/stat` and `status` over idle windows, peak RSS (`VmHWM`), binary size.

Nothing touched the maintainer's own config or terminal session. Every gate (fmt, typos,
`cargo deny check`, clippy and tests at the three feature levels, docs) passes on every
commit; regression tests were checked by stashing the fix, or by running them on a worktree
of the base.

## 2. Bugs

| Area | Finding | Status |
| ---- | ------- | ------ |
| Date formats | `date_format = "%Q %d"` (any directive chrono doesn't know) panicked on the first draw: chrono's `format` errors and `to_string` turns that into a panic. With the panic message also wiped (§4), calendar-tui vanished with exit 101 and no explanation. jiff prints unknown directives as written. | Fixed, high (`0238d37`, test `fcc782d`) |
| Year view, short terminals | Each mini month needs six week rows plus its border, so the year view (the default) needed 33 lines; below that the months that didn't fit were empty boxes: all twelve on an 80x24 terminal, the last row up to 32 lines. The layout now steps down: one status line instead of two boxes, then months without borders; below 92x23 a message points to the month view. | Fixed, high (`4ea5561`, `5dcfaf5`) |
| Year view, narrow terminals | Below ~110 columns, days got two-character columns and ran together (`28293031`), and the week number touched the first day (`526 27` for week 5, the 26th), even at 100 columns. The layout now also chooses 4, 3 or 2 months a row so every day has three columns; week numbers are dropped only if nothing fits with them. | Fixed, medium (`5dcfaf5`) |
| Theme colors | `parse_color` sliced the value into byte pairs after checking only its length: `"#aéééb"` is 8 bytes, the slice at byte 2 fell inside `é`, and calendar-tui panicked. `from_str_radix` also took a sign, so `"#+f+f+f"` parsed. | Fixed, medium (`c891315`) |
| Selection | In month view, `Space` selects today, but today's style was applied before the selection, so the first press seemed to do nothing. | Fixed, medium (`cba01f0`) |
| Today | "Today" was set at start-up and only refreshed by `t`; left open overnight, the calendar highlighted yesterday. It now moves at local midnight. | Fixed, medium (`e5c6a19`) |
| Year range | jiff's dates end at ±9999 (chrono's at ±262143); navigation now stops at ±9998 so every grid keeps its padding days. Tested at both ends for every week start. | Fixed, low (`0238d37`) |
| Tiny terminals | Both views at every size from 0x0 to 39x29: no panic. | OK |

## 3. Performance

Release builds, idle in a 120x40 tmux pane, three 20-second windows each:

| Measure | Base | Final |
| ------- | ---- | ----- |
| CPU time | 140 / 160 / 150 ms | 0 / 0 / 0 ms |
| Wakeups (voluntary context switches) | 198 / 198 / 198 | 0 / 0 / 0 |
| Peak RSS (`VmHWM`) | 4.79-4.85 MB | 4.73-4.82 MB |
| Binary size | 3.22 MB | 2.70 MB |
| Crates in the normal dependency tree | 116 | 109 |

| Area | Finding | Status |
| ---- | ------- | ------ |
| Event loop | Polled every 100 ms and redrew each time: ten wakeups and a full frame a second, all day, for a program meant to stay open. It now draws only after a key, a resize or a new day, and otherwise sleeps until midnight (at most a minute). | Fixed, medium (`e5c6a19`) |
| Logging dependencies | tracing-subscriber's `env-filter` pulled in regex-automata and matchers to read `RUST_LOG` in a one-crate program; `RUST_LOG` is now read as a level. Binary 3.60 MB (after the jiff switch) -> 2.69 MB. | Fixed, low (`49792de`) |
| Date library | jiff with only `std`, `tz-system` and `tzdb-zoneinfo` (the system's zoneinfo): +0.34 MB over chrono. | Accepted (the advisory, §4) |

Not measured: start-up time (it needs a terminal, and measure.py runs commands headless;
the program reads two small files before drawing) and key-to-frame latency (one frame, no
I/O).

## 4. Security and robustness

| Area | Finding | Status |
| ---- | ------- | ------ |
| Panic messages | A panic printed its message on the alternate screen; unwinding then left the alternate screen and wiped it, so any crash looked like the calendar silently closing with status 101. A panic hook now restores the terminal first, exactly once (leaving the alternate screen twice moved the cursor back over the message). If setting up the terminal fails half-way, raw mode is turned off again. | Fixed, high (`d3d61f9`) |
| Dependencies | `cargo deny check` failed: chrono is unmaintained (RUSTSEC-2026-0543). CI ran `cargo deny check licenses` only, so it never showed. Replaced with jiff; CI now runs every check, and daily. The ignore for RUSTSEC-2024-0436 (paste) matched nothing any more and is gone. | Fixed, medium (`0238d37`, `3556a5d`) |
| Output streams | Logs went through tracing-subscriber's default writer, stdout: errors landed in the program's output, with color codes, and stderr stayed empty. Now stderr, colored only on a terminal. (`49792de`'s message wrongly says the base already used stderr; found while writing `78bd73e`.) | Fixed, low (`49792de`, test `78bd73e`) |
| Terminal escapes | The only text drawn comes from the config (date formats) and the program itself; nothing from outside. | OK |
| Files | Read only, never written; paths come from the user. | OK |

## 5. Usability

| Area | Finding | Status |
| ---- | ------- | ------ |
| Unknown keys | Config and theme accepted any key: `weekstart = "sunday"` left the week on Monday, `[calender]` ignored the whole section. Now errors that name the key and file (exit 1). The maintainer's config and theme are still valid. | Fixed, medium (`2ebdfc2`) |
| Named files | `--config nope.toml` (or `--theme`, or the config's `theme` key) started with the defaults behind a warning the TUI covered. Named files must now exist; a missing file at the default location still means the defaults. | Fixed, medium (`6325e72`) |
| Start-up noise | With no config files, every start left two warnings and an INFO line with timestamps in the shell. Now silent; errors read `ERROR failed to read <path>: ...`. | Fixed, low (`6325e72`, `49792de`) |
| Color errors | Named only the failing pair (`invalid color zz`), not the key, value or file. | Fixed, low (`c891315`) |
| README | The configuration section stopped at "see below". It now covers every key, theme lookup, the command line, exit codes, logging and the year view's size needs. | Fixed, low (`c8eefec`) |
| Keybindings | README, help line and code agree. | OK |

## 6. Code quality, tests and hygiene

| Area | Finding | Status |
| ---- | ------- | ------ |
| Coverage | Lines (`cargo llvm-cov --all-features`): **72.3 % -> 90.0 %**. `app/` 0 % -> 60 % (the rest is the terminal loop), `main.rs` 0 % -> 68.6 %, `config/` 47 % -> 96 %. | Fixed (`35e7c2c`, `78bd73e`) |
| Integration tests | None. `tests/cli.rs` runs the binary with a scratch HOME and nothing else in its environment: exit codes, config errors, output streams; 6 of its 7 tests fail on the base. | Fixed (`78bd73e`) |
| Clock in tests | Two view tests read the clock twice and compared; across midnight they disagree. They use a fixed date now; new tests never read the clock. | Fixed, low (`35e7c2c`) |
| Flaky tests | rust-template's `stress.sh 20` (every test binary 20 times in parallel): 0 failures at the base and at the end. | OK |
| Gates | All passed at the base except `cargo deny check`; every audit commit passes all of them. | Fixed (`0238d37`) |
| CI | Clippy ran with all features and with none, not the defaults; deny compiled cargo-deny on every run. | Fixed, low (`3556a5d`) |
| Manifests | Every cargo command warned about the redundant `homepage` and unused workspace `description`. | Fixed, low (`3556a5d`) |
| Layout | `logger.rs` was the one module not in a folder with `tests.rs` (ARCHITECTURE.md §1.3). | Fixed (`49792de`) |

## 7. Left open

- **The `provider` feature.** Declared, empty, used nowhere; it looks reserved for the planned
  event sync (CHANGELOG 0.1.0 notes). Keep it as a marker, or remove it until it has code.
- **`Settings::from_cli` ignores its `Cli`.** It exists for the resolver pattern of
  ARCHITECTURE.md §3, but the command line only selects files, so nothing is merged. Harmless;
  it becomes real once a flag overrides a setting.
- **Year view below 92x23.** It shows a message. Two months a row needs 6 rows of months and
  doesn't help on short terminals; a scrolling year view would, at the cost of a new
  interaction. The month view fits much smaller terminals.
- **The help line on narrow terminals** is cut off rather than wrapped. Cosmetic.

## 8. Re-running the audit

```sh
# gates (rust-template's scripts/gates.sh runs the same set)
cargo fmt --all -- --check && typos && cargo deny check
for f in --no-default-features "" --all-features; do
  cargo clippy --workspace --all-targets $f -- -D warnings && cargo test --workspace $f
done
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features

# flaky tests and coverage
path/to/rust-template/scripts/stress.sh 20
cargo llvm-cov --all-features --summary-only

# idle cost: run the release binary in a tmux pane, then over 20 s compare
#   fields 14+15 of /proc/<pid>/stat (CPU ticks) and voluntary_ctxt_switches,
#   VmHWM in /proc/<pid>/status
# year view: capture the pane at 80x24, 80x40, 100x40 and 120x40
```
