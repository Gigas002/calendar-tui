use std::io::IsTerminal;

use tracing::level_filters::LevelFilter;

/// Shown when `RUST_LOG` isn't set: problems only. Logs go to stderr, which is the
/// terminal the calendar draws on, so everything printed before it starts stays in the
/// shell afterwards.
const DEFAULT_LEVEL: LevelFilter = LevelFilter::WARN;

pub fn init() {
    let level = level_from(std::env::var("RUST_LOG").ok().as_deref());
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .without_time()
        .with_target(false)
        .init();
}

/// The level in `RUST_LOG`: a plain level (`debug`), or the level of a `target=level`
/// directive, since this is one crate. Anything else means the default.
pub fn level_from(var: Option<&str>) -> LevelFilter {
    var.map(str::trim)
        .map(|v| v.rsplit('=').next().unwrap_or(v))
        // tracing parses "" as ERROR; an empty variable means unset.
        .filter(|v| !v.is_empty())
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_LEVEL)
}

#[cfg(test)]
mod tests;
