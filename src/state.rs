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

pub const YEAR_MIN: i32 = 1000;
pub const YEAR_MAX: i32 = 2500;

/// Sens de la lecture. `Stopped` laisse la date immobile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Playback {
    Stopped,
    Forward,
    Backward,
}

/// Unité du saut d'animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepUnit {
    Days,
    Months,
    Years,
}

impl StepUnit {
    pub fn label(self) -> &'static str {
        match self {
            StepUnit::Days => "Jours",
            StepUnit::Months => "Mois",
            StepUnit::Years => "Années",
        }
    }

    pub fn max_amount(self) -> i32 {
        match self {
            StepUnit::Days => 365,
            StepUnit::Months => 12,
            StepUnit::Years => 10,
        }
    }
}

pub struct Observation {
    pub when: DateTime<Local>,
    /// Si vrai, le nord céleste est affiché vers le bas (comme dans un télescope).
    pub north_down: bool,
    pub playback: Playback,
    pub step_amount: i32,
    pub step_unit: StepUnit,
    /// Instant où le prochain pas est dû. `None` quand la lecture est arrêtée.
    pub next_step: Option<std::time::Instant>,
}

impl Observation {
    pub fn new() -> Self {
        Self {
            when: truncate_minute(Local::now()),
            north_down: false,
            playback: Playback::Stopped,
            step_amount: 1,
            step_unit: StepUnit::Days,
            next_step: None,
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
