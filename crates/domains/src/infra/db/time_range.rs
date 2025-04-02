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
#[serde(rename_all = "camelCase")]
pub enum TimeRange {
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "12h")]
    TwelveHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
    #[serde(rename = "90d")]
    NinetyDays,
    #[serde(rename = "180d")]
    OneEightyDays,
    #[serde(rename = "1y")]
    OneYear,
    #[default]
    #[serde(rename = "all")]
    All,
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
            Self::OneHour => Some(Duration::hours(1)),
            Self::TwelveHours => Some(Duration::hours(12)),
            Self::OneDay => Some(Duration::days(1)),
            Self::SevenDays => Some(Duration::days(7)),
            Self::ThirtyDays => Some(Duration::days(30)),
            Self::NinetyDays => Some(Duration::days(90)),
            Self::OneEightyDays => Some(Duration::days(180)),
            Self::OneYear => Some(Duration::days(365)),
            Self::All => None,
        }
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::OneHour => "1h",
            Self::TwelveHours => "12h",
            Self::OneDay => "1d",
            Self::SevenDays => "7d",
            Self::ThirtyDays => "30d",
            Self::NinetyDays => "90d",
            Self::OneEightyDays => "180d",
            Self::OneYear => "1y",
            Self::All => "all",
        })
    }
}
