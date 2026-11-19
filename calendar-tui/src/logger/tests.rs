use tracing::level_filters::LevelFilter;

use super::level_from;

#[test]
fn defaults_to_warnings() {
    assert_eq!(level_from(None), LevelFilter::WARN);
    assert_eq!(level_from(Some("")), LevelFilter::WARN);
    assert_eq!(level_from(Some("loud")), LevelFilter::WARN);
}

#[test]
fn reads_a_level_or_a_directive() {
    assert_eq!(level_from(Some("debug")), LevelFilter::DEBUG);
    assert_eq!(level_from(Some(" TRACE ")), LevelFilter::TRACE);
    assert_eq!(level_from(Some("off")), LevelFilter::OFF);
    assert_eq!(level_from(Some("calendar_tui=info")), LevelFilter::INFO);
}
