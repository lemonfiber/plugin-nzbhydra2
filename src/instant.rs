use std::time::{Duration, SystemTime, UNIX_EPOCH};

use lemonfiber_manifest::Date;

const DAY: u64 = 86_400;

const HOUR: u64 = 3_600;

const MINUTE: u64 = 60;

pub(crate) fn written(at: SystemTime) -> Option<String> {
    let seconds = at.duration_since(UNIX_EPOCH).ok()?.as_secs();
    let date = Date::from_unix_seconds(i64::try_from(seconds).ok()?)?;
    let clock = seconds % DAY;
    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        date.year,
        date.month,
        date.day,
        clock / HOUR,
        (clock % HOUR) / MINUTE,
        clock % MINUTE
    ))
}

pub(crate) fn from_seconds(seconds: f64) -> Option<SystemTime> {
    (seconds.is_finite() && seconds >= 0.0)
        .then(|| UNIX_EPOCH + Duration::from_secs_f64(seconds.trunc()))
}

#[cfg(test)]
mod tests;
