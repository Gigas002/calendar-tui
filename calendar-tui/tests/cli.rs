//! Runs the real binary. Each test gets its own HOME and XDG config dir and nothing else in
//! its environment, so none reads or writes the user's config. stdout is a pipe, not a
//! terminal, so a run that gets past the config stops with exit 2 before drawing.

use std::path::Path;
use std::process::{Command, Output};

fn run(home: &Path, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_calendar-tui"));
    command
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"));
    // Under cargo-llvm-cov, where the instrumented binary writes its profile: without it the
    // profile lands in the working directory and the run isn't counted.
    if let Some(profile) = std::env::var_os("LLVM_PROFILE_FILE") {
        command.env("LLVM_PROFILE_FILE", profile);
    }
    command.output().expect("the calendar-tui binary runs")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn without_a_terminal_it_exits_2_and_says_why() {
    let home = tempfile::tempdir().unwrap();
    let output = run(home.path(), &[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).contains("not a terminal"),
        "{}",
        stderr(&output)
    );
    // No config at the default location is normal: no warnings about it.
    assert!(!stderr(&output).contains("WARN"), "{}", stderr(&output));
}

#[test]
fn reads_the_default_config_location() {
    let home = tempfile::tempdir().unwrap();
    let dir = home.path().join(".config/calendar-tui");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("config.toml"),
        "[calendar]\nweek_start = \"funday\"\n",
    )
    .unwrap();
    let output = run(home.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("config.toml"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_named_config_must_exist() {
    let home = tempfile::tempdir().unwrap();
    let missing = home.path().join("nope.toml");
    let output = run(home.path(), &["--config", missing.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("nope.toml"), "{}", stderr(&output));
}

#[test]
fn an_unknown_key_exits_1_and_is_named() {
    let home = tempfile::tempdir().unwrap();
    let config = home.path().join("config.toml");
    std::fs::write(&config, "[display]\ndefault_view = \"month\"\n").unwrap();
    let output = run(home.path(), &["--config", config.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("default_view"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_bad_theme_color_exits_1_naming_key_and_file() {
    let home = tempfile::tempdir().unwrap();
    let theme = home.path().join("theme.toml");
    std::fs::write(&theme, "[calendar]\ntoday = \"#aéééb\"\n").unwrap();
    let output = run(home.path(), &["--theme", theme.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let err = stderr(&output);
    assert!(
        err.contains("calendar.today") && err.contains("theme.toml"),
        "{err}"
    );
}

#[test]
fn the_examples_load() {
    let home = tempfile::tempdir().unwrap();
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples");
    let config = examples.join("config.toml");
    let output = run(home.path(), &["--config", config.to_str().unwrap()]);
    // Past config and theme (found next to the config): only the missing terminal stops it.
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
}

/// The base logged through tracing-subscriber's default writer, stdout: errors landed in
/// the program's output (with color codes), and stderr stayed empty.
#[test]
fn errors_go_to_stderr_without_color_codes() {
    let home = tempfile::tempdir().unwrap();
    let output = run(home.path(), &["--config", "/nonexistent/config.toml"]);
    assert!(
        output.stdout.is_empty(),
        "{:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    let err = stderr(&output);
    assert!(err.contains("/nonexistent/config.toml"), "{err:?}");
    assert!(!err.contains('\x1b'), "{err:?}");
}
