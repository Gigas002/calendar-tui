mod app;
mod error;
mod logger;

use std::process::ExitCode;

use error::Error;

fn main() -> ExitCode {
    logger::init();

    match app::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::NotATty) => {
            tracing::error!("{}", Error::NotATty);
            ExitCode::from(2)
        }
        Err(err) => {
            tracing::error!(%err, "application error");
            ExitCode::from(1)
        }
    }
}
