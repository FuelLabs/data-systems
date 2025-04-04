use chrono::{DateTime, Days, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::FuelCoreTai64;

#[derive(Debug, thiserror::Error)]
pub enum BlockTimestampError {
    #[error("Failed to convert TAI64 timestamp to DateTime")]
    InvalidTimestamp,
    #[error("Failed to parse timestamp string: {0}")]
    ParseError(String),
    #[error("Timestamp value out of range")]
    OutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct BlockTimestamp(pub DateTime<Utc>);

impl BlockTimestamp {
    pub fn from_unix_timestamp(
        timestamp: i64,
    ) -> Result<Self, BlockTimestampError> {
        Utc.timestamp_opt(timestamp, 0)
            .single()
            .map(Self)
            .ok_or(BlockTimestampError::OutOfRange)
    }

    pub fn from_tai64(tai: FuelCoreTai64) -> Result<Self, BlockTimestampError> {
        let unix_timestamp = tai.to_unix();
        Self::from_unix_timestamp(unix_timestamp)
            .map(|ts| Self(Self::normalize_to_micros(ts.0)))
    }

    pub fn unix_timestamp(&self) -> i64 {
        self.0.timestamp()
    }

    pub fn now() -> Self {
        Self(Self::normalize_to_micros(Utc::now()))
    }

    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(Self::normalize_to_micros(dt))
    }

    pub fn into_inner(self) -> DateTime<Utc> {
        self.0
    }

    pub fn as_inner(&self) -> &DateTime<Utc> {
        &self.0
    }

    pub fn from_secs(secs: i64) -> Self {
        Self(
            Utc.timestamp_opt(secs, 0)
                .single()
                .expect("invalid timestamp"),
        )
    }

    pub fn try_from_secs(secs: i64) -> Option<Self> {
        Utc.timestamp_opt(secs, 0).single().map(Self)
    }

    pub fn to_seconds(&self) -> i64 {
        self.0.timestamp()
    }

    pub fn is_after(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    pub fn is_before(&self, other: &Self) -> bool {
        self.0 < other.0
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    pub fn is_at_or_after(&self, other: &Self) -> bool {
        self.0 >= other.0
    }

    pub fn is_at_or_before(&self, other: &Self) -> bool {
        self.0 <= other.0
    }

    pub fn add(
        &self,
        value: i64,
        unit: TimeUnit,
    ) -> Result<Self, BlockTimestampError> {
        let seconds = match unit {
            TimeUnit::Seconds => value,
            TimeUnit::Minutes => value
                .checked_mul(60)
                .ok_or(BlockTimestampError::OutOfRange)?,
            TimeUnit::Hours => value
                .checked_mul(3600)
                .ok_or(BlockTimestampError::OutOfRange)?,
            TimeUnit::Days => value
                .checked_mul(86400)
                .ok_or(BlockTimestampError::OutOfRange)?,
        };

        let new_timestamp = self
            .unix_timestamp()
            .checked_add(seconds)
            .ok_or(BlockTimestampError::OutOfRange)?;
        Self::from_unix_timestamp(new_timestamp)
    }

    pub fn subtract(
        &self,
        value: i64,
        unit: TimeUnit,
    ) -> Result<Self, BlockTimestampError> {
        let seconds = match unit {
            TimeUnit::Seconds => value,
            TimeUnit::Minutes => value
                .checked_mul(60)
                .ok_or(BlockTimestampError::OutOfRange)?,
            TimeUnit::Hours => value
                .checked_mul(3600)
                .ok_or(BlockTimestampError::OutOfRange)?,
            TimeUnit::Days => value
                .checked_mul(86400)
                .ok_or(BlockTimestampError::OutOfRange)?,
        };

        let new_timestamp = self
            .unix_timestamp()
            .checked_sub(seconds)
            .ok_or(BlockTimestampError::OutOfRange)?;
        Self::from_unix_timestamp(new_timestamp)
    }

    pub fn diff_secs(&self, other: &Self) -> i64 {
        self.unix_timestamp() - other.unix_timestamp()
    }

    pub fn diff_ms(&self, other: &Self) -> i64 {
        self.0.timestamp_millis() - other.0.timestamp_millis()
    }

    pub fn is_between(&self, start: &Self, end: &Self) -> bool {
        self.is_at_or_after(start) && self.is_at_or_before(end)
    }

    pub fn is_strictly_between(&self, start: &Self, end: &Self) -> bool {
        self.is_after(start) && self.is_before(end)
    }

    pub fn is_within_days(&self, days: u32) -> bool {
        let now = Utc::now();
        let days = Days::new(days as u64);
        let days_ago = now.checked_sub_days(days);
        if let Some(days_ago) = days_ago {
            self.is_at_or_after(&days_ago.into())
        } else {
            false
        }
    }

    fn normalize_to_micros(dt: DateTime<Utc>) -> DateTime<Utc> {
        let micros = dt.timestamp_micros();
        Utc.timestamp_micros(micros).single().unwrap_or(dt)
    }
}

impl Default for BlockTimestamp {
    fn default() -> Self {
        Self(Utc::now())
    }
}

impl utoipa::ToSchema for BlockTimestamp {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("BlockTimestamp")
    }
}

impl utoipa::PartialSchema for BlockTimestamp {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::schema::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::Integer)
            .format(Some(utoipa::openapi::schema::SchemaFormat::Custom(
                "unix-timestamp".to_string(),
            )))
            .description(Some("Block timestamp as Unix seconds since epoch"))
            .examples([Some(serde_json::json!(Utc::now().timestamp()))])
            .build()
            .into()
    }
}

impl From<&super::BlockHeader> for BlockTimestamp {
    fn from(header: &super::BlockHeader) -> Self {
        Self::from_tai64(header.time.clone().into_inner())
            .unwrap_or_else(|_| Self(Utc::now()))
    }
}

impl std::fmt::Display for BlockTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.unix_timestamp())
    }
}

impl std::str::FromStr for BlockTimestamp {
    type Err = BlockTimestampError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<i64>()
            .map_err(|e| BlockTimestampError::ParseError(e.to_string()))
            .and_then(Self::from_unix_timestamp)
    }
}

impl From<DateTime<Utc>> for BlockTimestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self::new(dt)
    }
}

impl AsRef<DateTime<Utc>> for BlockTimestamp {
    fn as_ref(&self) -> &DateTime<Utc> {
        &self.0
    }
}

impl Serialize for BlockTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.unix_timestamp().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BlockTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        Self::from_unix_timestamp(timestamp)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for BlockTimestamp {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let dt =
            <chrono::DateTime<Utc> as sqlx::Decode<sqlx::Postgres>>::decode(
                value,
            )?;
        Ok(Self(dt))
    }
}

impl sqlx::Type<sqlx::Postgres> for BlockTimestamp {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <chrono::DateTime<Utc> as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for BlockTimestamp {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<
        sqlx::encode::IsNull,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {
        <chrono::DateTime<Utc> as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(
            &self.0, buf,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Duration;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{BlockHeader, BlockTime};

    #[test]
    fn test_from_unix_timestamp() {
        // Test valid timestamp
        let timestamp =
            BlockTimestamp::from_unix_timestamp(1234567890).unwrap();
        assert_eq!(timestamp.unix_timestamp(), 1234567890);

        // Test timestamp at unix epoch
        let timestamp = BlockTimestamp::from_unix_timestamp(0).unwrap();
        assert_eq!(timestamp.unix_timestamp(), 0);

        // Test invalid timestamp (too far in the future or past)
        assert!(BlockTimestamp::from_unix_timestamp(i64::MAX).is_err());
        assert!(BlockTimestamp::from_unix_timestamp(i64::MIN).is_err());
    }

    #[test]
    fn test_from_str() {
        // Test valid string
        let timestamp = BlockTimestamp::from_str("1234567890").unwrap();
        assert_eq!(timestamp.unix_timestamp(), 1234567890);

        // Test invalid string
        assert!(BlockTimestamp::from_str("invalid").is_err());
        assert!(BlockTimestamp::from_str("").is_err());
    }

    #[test]
    fn test_display() {
        let timestamp =
            BlockTimestamp::from_unix_timestamp(1234567890).unwrap();
        assert_eq!(timestamp.to_string(), "1234567890");
    }

    #[test]
    fn test_tai64_conversion() {
        // Assuming a valid TAI64 timestamp
        let tai = FuelCoreTai64::from_unix(1234567890);
        let timestamp = BlockTimestamp::from_tai64(tai).unwrap();
        assert_eq!(timestamp.unix_timestamp(), 1234567890);
    }

    #[test]
    fn test_timestamp_conversions() {
        let now = Utc::now();

        // Test new() and into_inner()
        let timestamp = BlockTimestamp::new(now);
        assert_eq!(timestamp.into_inner(), now);

        // Test as_inner()
        let timestamp = BlockTimestamp::new(now);
        assert_eq!(timestamp.as_inner(), &now);

        // Test from_secs() and try_from_secs()
        let secs = 1234567890;
        let timestamp = BlockTimestamp::from_secs(secs);
        assert_eq!(timestamp.unix_timestamp(), secs);

        let opt_timestamp = BlockTimestamp::try_from_secs(secs);
        assert!(opt_timestamp.is_some());
        assert_eq!(opt_timestamp.unwrap().unix_timestamp(), secs);

        // Test invalid timestamp
        let opt_timestamp = BlockTimestamp::try_from_secs(i64::MAX);
        assert!(opt_timestamp.is_none());

        // Test From<DateTime<Utc>>
        let timestamp: BlockTimestamp = now.into();
        assert_eq!(timestamp.as_inner(), &now);

        // Test AsRef<DateTime<Utc>>
        let timestamp = BlockTimestamp::new(now);
        assert_eq!(timestamp.as_ref(), &now);
    }

    #[test]
    fn test_serde_timestamp() {
        use serde_json;

        // Test serialization to integer
        let timestamp =
            BlockTimestamp::from_unix_timestamp(1234567890).unwrap();
        let serialized = serde_json::to_string(&timestamp).unwrap();
        assert_eq!(serialized, "1234567890");

        // Test deserialization from integer
        let deserialized: BlockTimestamp =
            serde_json::from_str("1234567890").unwrap();
        assert_eq!(deserialized.unix_timestamp(), 1234567890);

        // Test deserialization error
        let result: Result<BlockTimestamp, _> =
            serde_json::from_str("\"invalid\"");
        assert!(result.is_err());

        // Test deserialization of out-of-range value
        let result: Result<BlockTimestamp, _> =
            serde_json::from_str(&i64::MAX.to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_query_string_parsing() {
        use serde_urlencoded;

        // Test parsing from query string
        let query_string = "timestamp=1234567890";
        let parsed: std::collections::HashMap<String, BlockTimestamp> =
            serde_urlencoded::from_str(query_string).unwrap();
        assert_eq!(parsed["timestamp"].unix_timestamp(), 1234567890);

        // Test invalid query string
        let query_string = "timestamp=invalid";
        let parsed: Result<
            std::collections::HashMap<String, BlockTimestamp>,
            _,
        > = serde_urlencoded::from_str(query_string);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_block_conversion() {
        // Create a mock block with known timestamp
        let unix_time = 1234567890;
        let mut block_header = BlockHeader::default();
        let time = BlockTime::from(FuelCoreTai64::from_unix(unix_time));
        block_header.time = time.clone();
        let timestamp = BlockTimestamp::from(&block_header);
        assert_eq!(timestamp.unix_timestamp(), unix_time);
    }

    #[test]
    fn test_default() {
        // Test that default is within a reasonable range of now
        let before = Utc::now();
        let timestamp = BlockTimestamp::default();
        let after = Utc::now();

        let timestamp_unix = timestamp.unix_timestamp();
        let before_unix = before.timestamp();
        let after_unix = after.timestamp();

        assert!(
            timestamp_unix >= before_unix && timestamp_unix <= after_unix,
            "Default timestamp {} should be between {} and {}",
            timestamp_unix,
            before_unix,
            after_unix
        );
    }

    #[test]
    fn test_comparison_methods() {
        // Create timestamps with different values
        let earlier = BlockTimestamp::from_unix_timestamp(1000000000).unwrap();
        let later = BlockTimestamp::from_unix_timestamp(1500000000).unwrap();

        // Test is_greater
        assert!(!earlier.is_after(&later));
        assert!(later.is_after(&earlier));

        // Test is_less
        assert!(earlier.is_before(&later));
        assert!(!later.is_before(&earlier));

        // Test with equal timestamps
        let same1 = BlockTimestamp::from_unix_timestamp(1234567890).unwrap();
        let same2 = BlockTimestamp::from_unix_timestamp(1234567890).unwrap();

        assert!(!same1.is_after(&same2));
        assert!(!same1.is_before(&same2));
    }

    #[test]
    fn test_equality_methods() {
        let ts1 = BlockTimestamp::from_unix_timestamp(1234567890).unwrap();
        let ts2 = BlockTimestamp::from_unix_timestamp(1234567890).unwrap();
        let ts3 = BlockTimestamp::from_unix_timestamp(1234567891).unwrap();

        assert!(ts1.is_equal(&ts2));
        assert!(!ts1.is_equal(&ts3));

        assert!(ts1.is_at_or_after(&ts2));
        assert!(ts3.is_at_or_after(&ts1));
        assert!(!ts1.is_at_or_after(&ts3));

        assert!(ts1.is_at_or_before(&ts2));
        assert!(ts1.is_at_or_before(&ts3));
        assert!(!ts3.is_at_or_before(&ts1));
    }

    #[test]
    fn test_time_arithmetic_with_units() {
        let ts = BlockTimestamp::from_unix_timestamp(1000).unwrap();

        // Test adding with different units
        let ts_plus_seconds = ts.add(500, TimeUnit::Seconds).unwrap();
        assert_eq!(ts_plus_seconds.unix_timestamp(), 1500);

        let ts_plus_minutes = ts.add(5, TimeUnit::Minutes).unwrap();
        assert_eq!(ts_plus_minutes.unix_timestamp(), 1300); // 1000 + 5*60

        let ts_plus_hours = ts.add(1, TimeUnit::Hours).unwrap();
        assert_eq!(ts_plus_hours.unix_timestamp(), 4600); // 1000 + 1*3600

        let ts_plus_days = ts.add(1, TimeUnit::Days).unwrap();
        assert_eq!(ts_plus_days.unix_timestamp(), 87400); // 1000 + 1*86400

        // Test subtracting with different units
        let ts_minus_seconds = ts.subtract(500, TimeUnit::Seconds).unwrap();
        assert_eq!(ts_minus_seconds.unix_timestamp(), 500);

        let ts_minus_minutes = ts.subtract(5, TimeUnit::Minutes).unwrap();
        assert_eq!(ts_minus_minutes.unix_timestamp(), 700); // 1000 - 5*60

        let ts_minus_hours = ts.subtract(0, TimeUnit::Hours).unwrap();
        assert_eq!(ts_minus_hours.unix_timestamp(), 1000); // 1000 - 0*3600

        // Test overflow/underflow handling
        assert!(ts.add(i64::MAX, TimeUnit::Days).is_err());
        assert!(ts.subtract(i64::MAX, TimeUnit::Hours).is_err());
    }

    #[test]
    fn test_difference_calculation_secs() {
        let earlier = BlockTimestamp::from_unix_timestamp(1000).unwrap();
        let later = BlockTimestamp::from_unix_timestamp(1500).unwrap();

        assert_eq!(later.diff_secs(&earlier), 500);
        assert_eq!(earlier.diff_secs(&later), -500);
    }

    #[test]
    fn test_difference_calculation_ms() {
        let earlier = BlockTimestamp::from_unix_timestamp(1000).unwrap();
        let later = BlockTimestamp::from_unix_timestamp(1500).unwrap();

        assert_eq!(later.diff_ms(&earlier), 500000);
        assert_eq!(earlier.diff_ms(&later), -500000);
    }

    #[test]
    fn test_range_checking() {
        let start = BlockTimestamp::from_unix_timestamp(1000).unwrap();
        let middle = BlockTimestamp::from_unix_timestamp(1500).unwrap();
        let end = BlockTimestamp::from_unix_timestamp(2000).unwrap();

        // Test inclusive range
        assert!(middle.is_between(&start, &end));
        assert!(start.is_between(&start, &end));
        assert!(end.is_between(&start, &end));
        assert!(!BlockTimestamp::from_unix_timestamp(999)
            .unwrap()
            .is_between(&start, &end));
        assert!(!BlockTimestamp::from_unix_timestamp(2001)
            .unwrap()
            .is_between(&start, &end));

        // Test exclusive range
        assert!(middle.is_strictly_between(&start, &end));
        assert!(!start.is_strictly_between(&start, &end));
        assert!(!end.is_strictly_between(&start, &end));
    }

    #[test]
    fn test_is_within_days() {
        let now = Utc::now();
        let timestamp = BlockTimestamp::new(now);

        // Current timestamp should be within any positive number of days
        assert!(timestamp.is_within_days(1));
        assert!(timestamp.is_within_days(7));
        assert!(timestamp.is_within_days(30));

        // Test timestamp from 5 days ago
        let five_days_ago = now - Duration::days(5);
        let old_timestamp = BlockTimestamp::new(five_days_ago);

        assert!(old_timestamp.is_within_days(7)); // Should be within 7 days
        assert!(!old_timestamp.is_within_days(3)); // Should not be within 3 days

        // Test timestamp from future
        let future = now + Duration::days(1);
        let future_timestamp = BlockTimestamp::new(future);
        assert!(future_timestamp.is_within_days(7)); // Future timestamps count as within range

        // Test very old timestamp
        let very_old = now - Duration::days(100);
        let very_old_timestamp = BlockTimestamp::new(very_old);
        assert!(!very_old_timestamp.is_within_days(30)); // Should not be within 30 days
    }

    #[test]
    fn test_timestamp_precision_normalization() {
        // Create a timestamp with nanosecond precision
        let nano_precise =
            Utc.timestamp_opt(1234567890, 123456789).single().unwrap();
        let ts = BlockTimestamp::new(nano_precise);

        // The normalized timestamp should only have microsecond precision
        // 123456789 nanoseconds should be truncated to 123456 microseconds
        let expected_micros = Utc
            .timestamp_micros(nano_precise.timestamp_micros())
            .single()
            .unwrap();
        assert_eq!(ts.0, expected_micros);

        // Verify string representation has only 6 decimal places
        let formatted = format!("{:?}", ts.0);
        assert!(formatted.contains(".123456Z")); // Should only show microseconds
        assert!(!formatted.contains(".123456789")); // Should not contain nanoseconds
    }

    #[test]
    fn test_tai64_timestamp_precision() {
        // Create a TAI64 timestamp
        let tai = FuelCoreTai64::from_unix(1234567890);
        let ts = BlockTimestamp::from_tai64(tai).unwrap();

        // Verify the timestamp has microsecond precision
        let formatted = format!("{:?}", ts.0);
        assert!(!formatted.contains("789Z")); // Should not have nanosecond precision
    }

    #[test]
    fn test_now_timestamp_precision() {
        let ts = BlockTimestamp::now();

        // Get the string representation
        let formatted = format!("{:?}", ts.0);

        // Count the decimal places after the dot and before Z
        let decimal_places = formatted
            .split('.')
            .nth(1)
            .map(|s| s.trim_end_matches('Z').len())
            .unwrap_or(0);

        // Should have exactly 6 decimal places (microsecond precision)
        assert_eq!(
            decimal_places, 6,
            "Timestamp should have microsecond precision (6 decimal places)"
        );
    }
}
