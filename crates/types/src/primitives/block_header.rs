use std::{collections::HashMap, fmt, str::FromStr};

use apache_avro::{
    schema::{derive::AvroSchemaComponent, Name},
    AvroSchema,
    Schema,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::{
    de::{self, Deserializer, Visitor},
    ser::Serializer,
    Deserialize,
    Serialize,
};
use wrapped_int::WrappedU32;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockTime(pub FuelCoreTai64);

impl BlockTime {
    pub fn into_inner(self) -> FuelCoreTai64 {
        self.0
    }
    pub fn from_unix(secs: i64) -> Self {
        Self(FuelCoreTai64::from_unix(secs))
    }
}

impl From<FuelCoreTai64> for BlockTime {
    fn from(value: FuelCoreTai64) -> Self {
        Self(value)
    }
}

impl Default for BlockTime {
    fn default() -> Self {
        Self(FuelCoreTai64::from_unix(0))
    }
}

impl Serialize for BlockTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let unix_timestamp = self.0.to_unix();
        serializer.serialize_i64(unix_timestamp)
    }
}

impl<'de> Deserialize<'de> for BlockTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BlockTimeVisitor;

        impl<'de> Visitor<'de> for BlockTimeVisitor {
            type Value = BlockTime;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a string Unix timestamp or an 8-byte array or an integer",
                )
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(BlockTime(FuelCoreTai64::from_unix(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(BlockTime(FuelCoreTai64::from_unix(value as i64)))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let unix_timestamp =
                    value.parse::<i64>().map_err(de::Error::custom)?;
                Ok(BlockTime(FuelCoreTai64::from_unix(unix_timestamp)))
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let tai64 = FuelCoreTai64::from_slice(value).map_err(|_| {
                    de::Error::custom("expected an 8-byte array for TAI64")
                })?;
                Ok(BlockTime(tai64))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut bytes = [0u8; 8];
                for byte in &mut bytes {
                    *byte = seq.next_element()?.ok_or_else(|| {
                        de::Error::custom("byte array too short")
                    })?;
                }
                if seq.next_element::<u8>()?.is_some() {
                    return Err(de::Error::custom("byte array too long"));
                }
                let tai64 = FuelCoreTai64::from(bytes);
                Ok(BlockTime(tai64))
            }
        }

        deserializer.deserialize_any(BlockTimeVisitor)
    }
}

impl utoipa::ToSchema for BlockTime {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("BlockTime")
    }
}

impl utoipa::PartialSchema for BlockTime {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::schema::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::Integer)
            .format(Some(utoipa::openapi::schema::SchemaFormat::Custom(
                "tai64-timestamp".to_string(),
            )))
            .description(Some(
                "Block time as TAI64 format (convertible to/from Unix seconds)",
            ))
            .examples([Some(serde_json::json!(FuelCoreTai64::from_unix(
                Utc::now().timestamp()
            )))])
            .build()
            .into()
    }
}

impl AvroSchemaComponent for BlockTime {
    fn get_schema_in_ctxt(
        _ctxt: &mut HashMap<Name, Schema>,
        _namespace: &Option<String>,
    ) -> Schema {
        Schema::Long
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Default,
    utoipa::ToSchema,
    AvroSchema,
)]
#[serde(rename_all = "snake_case")]
pub struct BlockHeader {
    pub application_hash: Bytes32,
    pub consensus_parameters_version: WrappedU32,
    pub da_height: DaBlockHeight,
    pub event_inbox_root: Bytes32,
    pub id: BlockId,
    pub height: BlockHeight,
    pub message_outbox_root: Bytes32,
    pub message_receipt_count: WrappedU32,
    pub prev_root: Bytes32,
    pub state_transition_bytecode_version: WrappedU32,
    pub time: BlockTime,
    pub transactions_count: u16,
    pub transactions_root: Bytes32,
    pub version: BlockVersion,
}

impl BlockHeader {
    pub fn get_timestamp_utc(&self) -> DateTime<Utc> {
        let t = self.time.0;
        let tai64_timestamp = FuelCoreTai64Timestamp(t);
        let unix_timestamp = tai64_timestamp.to_unix();
        Utc.timestamp_opt(unix_timestamp, 0).unwrap()
    }
}

impl From<&FuelCoreBlockHeader> for BlockHeader {
    fn from(header: &FuelCoreBlockHeader) -> Self {
        let version = match header {
            FuelCoreBlockHeader::V1(_) => BlockVersion::V1,
        };

        Self {
            application_hash: (*header.application_hash()).into(),
            consensus_parameters_version: header
                .consensus_parameters_version()
                .into(),
            da_height: header.da_height().into(),
            event_inbox_root: header.event_inbox_root().into(),
            id: header.id().into(),
            height: (*header.height()).into(),
            message_outbox_root: header.message_outbox_root().into(),
            message_receipt_count: header.message_receipt_count().into(),
            prev_root: (*header.prev_root()).into(),
            state_transition_bytecode_version: header
                .state_transition_bytecode_version()
                .into(),
            time: header.time().into(),
            transactions_count: header.transactions_count(),
            transactions_root: header.transactions_root().into(),
            version,
        }
    }
}

// BlockVersion enum
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
    apache_avro::AvroSchema,
    Default,
)]
pub enum BlockVersion {
    #[default]
    #[serde(alias = "v1")]
    V1,
}

impl FromStr for BlockVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "V1" => Ok(BlockVersion::V1),
            _ => Err(format!("Unknown BlockVersion: {}", s)),
        }
    }
}

impl fmt::Display for BlockVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BlockVersion::V1 => "V1",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json;

    use super::*;

    #[test]
    fn test_serialize_string() {
        let block_time = BlockTime::from_unix(1614556800);
        let serialized = serde_json::to_string(&block_time).unwrap();
        assert_eq!(serialized, "1614556800"); // Expect integer
    }

    #[test]
    fn test_deserialize_string() {
        let json = "\"1614556800\"";
        let deserialized: BlockTime = serde_json::from_str(json).unwrap();
        let expected = BlockTime::from_unix(1614556800);
        assert_eq!(deserialized, expected);
        assert_eq!(deserialized.0.to_unix(), 1614556800);
    }

    #[test]
    fn test_deserialize_byte_array() {
        let block_time = BlockTime::from_unix(1614556800);
        let bytes = block_time.0.to_bytes();
        let json = serde_json::to_string(&bytes.to_vec()).unwrap();
        let deserialized: BlockTime = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, block_time);
        assert_eq!(deserialized.0.to_unix(), 1614556800);
    }

    #[test]
    fn test_round_trip_string() {
        let original = BlockTime::from_unix(1614556800);
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: BlockTime =
            serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn test_deserialize_invalid_string() {
        let json = "\"not-a-number\"";
        let result = serde_json::from_str::<BlockTime>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_invalid_byte_array_length() {
        let json = "[1, 2, 3]";
        let result = serde_json::from_str::<BlockTime>(json);
        assert!(result.is_err());

        let json = "[1, 2, 3, 4, 5, 6, 7, 8, 9]";
        let result = serde_json::from_str::<BlockTime>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_timestamp_utc() {
        let block_time = BlockTime::from_unix(1614556800);
        let header = BlockHeader {
            time: block_time,
            ..Default::default()
        };
        let utc_time = header.get_timestamp_utc();
        assert_eq!(utc_time.to_rfc3339(), "2021-03-01T00:00:00+00:00");
    }

    #[test]
    fn test_deserialize_integer() {
        let json = "1614556800";
        let deserialized: BlockTime = serde_json::from_str(json).unwrap();
        let expected = BlockTime::from_unix(1614556800);
        assert_eq!(deserialized, expected);
        assert_eq!(deserialized.0.to_unix(), 1614556800);
    }

    #[test]
    fn test_block_version_deserialization() {
        // Test uppercase "V1"
        let uppercase = r#""V1""#;
        let version: BlockVersion = serde_json::from_str(uppercase).unwrap();
        assert_eq!(version, BlockVersion::V1);

        // Test lowercase "v1"
        let lowercase = r#""v1""#;
        let version: BlockVersion = serde_json::from_str(lowercase).unwrap();
        assert_eq!(version, BlockVersion::V1);

        // Test within a JSON object
        let json_obj = serde_json::json!({
            "version": "V1"
        });
        let parsed: serde_json::Value =
            serde_json::from_value(json_obj).unwrap();
        let version: BlockVersion =
            serde_json::from_value(parsed["version"].clone()).unwrap();
        assert_eq!(version, BlockVersion::V1);
    }
}
