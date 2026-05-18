use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("standard output is not a terminal; run calendar-tui in a TTY")]
    NotATty,

    #[error("terminal error: {0}")]
    Terminal(#[from] io::Error),
}
