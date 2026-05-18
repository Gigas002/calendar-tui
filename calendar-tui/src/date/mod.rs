use chrono::{Datelike, Local, NaiveDate, Weekday};

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
            Self::Monday => Weekday::Mon,
            Self::Tuesday => Weekday::Tue,
            Self::Wednesday => Weekday::Wed,
            Self::Thursday => Weekday::Thu,
            Self::Friday => Weekday::Fri,
            Self::Saturday => Weekday::Sat,
            Self::Sunday => Weekday::Sun,
        }
    }

    /// Offset from Monday for column 0 alignment (Monday = 0, Sunday = 6).
    pub fn days_from_monday(self) -> u32 {
        self.weekday().num_days_from_monday()
    }

    /// Column index (0–6) for `weekday` when this week start is column 0.
    pub fn column_for(self, weekday: Weekday) -> u32 {
        let delta = weekday.num_days_from_monday() as i32 - self.days_from_monday() as i32;
        (((delta % 7) + 7) % 7) as u32
    }
}

pub fn today_local() -> NaiveDate {
    Local::now().date_naive()
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_of_next =
        NaiveDate::from_ymd_opt(next_year, next_month, 1).expect("valid year/month for next month");
    first_of_next
        .pred_opt()
        .expect("day before month end")
        .day()
}

pub fn naive_from_ymd(year: i32, month: u32, day: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(year, month, day)
}

#[cfg(test)]
mod tests;
