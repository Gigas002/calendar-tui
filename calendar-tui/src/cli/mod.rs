use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about = "Interactive terminal calendar")]
pub struct Cli {
    #[arg(
        long,
        value_name = "FILE",
        help = "Path to config.toml (default: XDG …/calendar-tui/config.toml)"
    )]
    pub config: Option<PathBuf>,

    #[arg(
        long,
        value_name = "FILE",
        help = "Path to theme.toml (default: resolved from [calendar].theme)"
    )]
    pub theme: Option<PathBuf>,
}

#[cfg(test)]
mod tests;
