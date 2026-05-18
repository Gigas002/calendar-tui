use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("standard output is not a terminal; run calendar-tui in a TTY")]
    NotATty,

    #[error("terminal error: {0}")]
    Terminal(#[from] io::Error),

    #[error("failed to read {path}: {source}")]
    ReadConfig {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse TOML at {path}: {source}")]
    TomlParse {
        path: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("invalid color {value}: {reason}")]
    InvalidColor { value: String, reason: String },
}
