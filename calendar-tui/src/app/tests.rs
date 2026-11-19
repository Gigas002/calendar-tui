use super::App;
use crate::config::Config;
use crate::date::naive_from_ymd;
use crate::settings::Settings;
use crate::theme::Theme;

fn app() -> App {
    App::new(Settings::resolve(&Config::default(), &Theme::default()).unwrap())
}

/// "Today" was set once at start-up, so a calendar left open past midnight kept highlighting
/// yesterday.
#[test]
fn set_today_moves_today_once() {
    let mut app = app();
    let tomorrow = app.settings.view.today.tomorrow().unwrap();
    assert!(app.set_today(tomorrow));
    assert_eq!(app.settings.view.today, tomorrow);
    assert!(!app.set_today(tomorrow), "unchanged date: no redraw");
    let other = naive_from_ymd(2030, 1, 1).unwrap();
    assert!(app.set_today(other));
}
