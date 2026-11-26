//! Instant d'observation partagé par tous les panneaux.

use chrono::{DateTime, Datelike, Local, LocalResult, NaiveDate, TimeZone, Timelike, Utc};

pub const MONTHS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

pub struct Observation {
    pub when: DateTime<Local>,
    /// Nord céleste vers le bas, comme dans un télescope sans renvoi coudé.
    pub north_down: bool,
}

impl Observation {
    pub fn new() -> Self {
        Self {
            when: truncate_minute(Local::now()),
            north_down: true,
        }
    }
}

pub fn truncate_minute(when: DateTime<Local>) -> DateTime<Local> {
    when.with_second(0)
        .and_then(|t| t.with_nanosecond(0))
        .unwrap_or(when)
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let Some(start) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return 30;
    };
    let Some(next) = NaiveDate::from_ymd_opt(next_year, next_month, 1) else {
        return 30;
    };
    (next - start).num_days().max(1) as u32
}

/// Reconstruit l'instant local. Un jour impossible (31 février) est ramené au dernier jour du mois.
pub fn with_civil(
    current: DateTime<Local>,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
) -> DateTime<Local> {
    let month = month.clamp(1, 12);
    let hour = hour.min(23);
    let minute = minute.min(59);
    let day = day.clamp(1, days_in_month(year, month));
    let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
        return current;
    };
    let Some(naive) = date.and_hms_opt(hour, minute, 0) else {
        return current;
    };
    match Local.from_local_datetime(&naive) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(earliest, _) => earliest,
        LocalResult::None => current,
    }
}

pub fn format_civil(when: DateTime<impl TimeZone>) -> String {
    format!(
        "{} {} {}, {:02} h {:02}",
        when.day(),
        MONTHS[when.month0() as usize],
        when.year(),
        when.hour(),
        when.minute()
    )
}

pub fn format_local(when: DateTime<Local>) -> String {
    format_civil(when)
}

pub fn format_utc(when: DateTime<Local>) -> String {
    format_civil(when.with_timezone(&Utc))
}
