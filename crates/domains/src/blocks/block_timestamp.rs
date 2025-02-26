use chrono::{DateTime, TimeZone, Utc};
use fuel_streams_types::FuelCoreTai64Timestamp;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum BlockTimestampError {
    #[error("Failed to convert TAI64 timestamp to DateTime")]
    InvalidTimestamp,
    #[error("Failed to parse timestamp string: {0}")]
    ParseError(String),
    #[error("Timestamp value out of range")]
    OutOfRange,
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

    pub fn from_tai64(
        tai: FuelCoreTai64Timestamp,
    ) -> Result<Self, BlockTimestampError> {
        let unix_timestamp = tai.to_unix();
        Self::from_unix_timestamp(unix_timestamp)
    }

    pub fn unix_timestamp(&self) -> i64 {
        self.0.timestamp()
    }

    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt)
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
}

impl Default for BlockTimestamp {
    fn default() -> Self {
        Self(Utc::now())
    }
}

impl From<&super::Block> for BlockTimestamp {
    fn from(block: &super::Block) -> Self {
        Self::from_tai64(block.header.time.clone())
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
        Self(dt)
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::assert_eq;

    use super::*;

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
        let tai = FuelCoreTai64Timestamp::from_unix(1234567890);
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
        use crate::mocks::MockBlock;

        // Create a mock block with known timestamp
        let unix_time = 1234567890;
        let block = MockBlock::build_with_timestamp(1, unix_time);

        // Test conversion from block
        let timestamp = BlockTimestamp::from(&block);
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
}
