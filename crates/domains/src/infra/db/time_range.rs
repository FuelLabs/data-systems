use std::fmt;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Default,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TimeRange {
    #[default]
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "30m")]
    ThirtyMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "12h")]
    TwelveHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "7d")]
    SevenDays,
}

impl TimeRange {
    pub fn time_since_now(&self) -> DateTime<Utc> {
        let now = Utc::now();
        if let Some(duration) = self.to_duration() {
            now - duration
        } else {
            now
        }
    }

    pub fn to_duration(&self) -> Option<Duration> {
        match self {
            Self::FiveMinutes => Some(Duration::minutes(5)),
            Self::FifteenMinutes => Some(Duration::minutes(15)),
            Self::ThirtyMinutes => Some(Duration::minutes(30)),
            Self::OneHour => Some(Duration::hours(1)),
            Self::TwelveHours => Some(Duration::hours(12)),
            Self::OneDay => Some(Duration::days(1)),
            Self::SevenDays => Some(Duration::days(7)),
        }
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::ThirtyMinutes => "30m",
            Self::OneHour => "1h",
            Self::TwelveHours => "12h",
            Self::OneDay => "1d",
            Self::SevenDays => "7d",
        })
    }
}
