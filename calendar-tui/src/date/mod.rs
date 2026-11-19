pub use jiff::civil::{Date, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WeekStart {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl WeekStart {
    #[allow(dead_code)] // used in unit tests
    pub const ALL: [WeekStart; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];

    pub fn weekday(self) -> Weekday {
        match self {
            Self::Monday => Weekday::Monday,
            Self::Tuesday => Weekday::Tuesday,
            Self::Wednesday => Weekday::Wednesday,
            Self::Thursday => Weekday::Thursday,
            Self::Friday => Weekday::Friday,
            Self::Saturday => Weekday::Saturday,
            Self::Sunday => Weekday::Sunday,
        }
    }

    /// Offset from Monday for column 0 alignment (Monday = 0, Sunday = 6).
    pub fn days_from_monday(self) -> u32 {
        self.weekday().to_monday_zero_offset() as u32
    }

    /// Column index (0–6) for `weekday` when this week start is column 0.
    pub fn column_for(self, weekday: Weekday) -> u32 {
        let delta = weekday.to_monday_zero_offset() as i32 - self.days_from_monday() as i32;
        (((delta % 7) + 7) % 7) as u32
    }
}

/// First and last year the calendar navigates to: one inside jiff's civil date range
/// (-9999..=9999), so every month grid has the neighbouring days that pad it.
pub const MIN_YEAR: i16 = -9998;
pub const MAX_YEAR: i16 = 9998;

pub fn today_local() -> Date {
    jiff::ZonedDateTime::now().date()
}

pub fn days_in_month(year: i16, month: i8) -> i8 {
    Date::new(year, month, 1)
        .expect("valid year/month")
        .days_in_month()
}

pub fn naive_from_ymd(year: i16, month: i8, day: i8) -> Option<Date> {
    Date::new(year, month, day).ok()
}

#[cfg(test)]
mod tests;
