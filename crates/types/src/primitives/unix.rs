use std::collections::HashMap;

use apache_avro::{
    schema::{derive::AvroSchemaComponent, Name},
    Schema,
};
use fuel_core_client::client::schema::Tai64Timestamp as FuelCoreTai64Timestamp;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tai64::Tai64;

pub use crate::primitives::BlockHeight;
use crate::primitives::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UnixTimestamp(pub Amount);

impl Serialize for UnixTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let tai64_timestamp = FuelCoreTai64Timestamp(tai64::Tai64(*self.0));
        let unix_timestamp = tai64_timestamp.to_unix();
        serializer.serialize_i64(unix_timestamp)
    }
}

impl<'de> Deserialize<'de> for UnixTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl serde::de::Visitor<'_> for ValueVisitor {
            type Value = UnixTimestamp;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                formatter.write_str("a string containing a number or a number")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let timestamp =
                    value.parse::<i64>().map_err(serde::de::Error::custom)?;
                let tai64_time =
                    FuelCoreTai64Timestamp(Tai64(timestamp as u64));
                Ok(UnixTimestamp(tai64_time.0 .0.into()))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let tai64_time = FuelCoreTai64Timestamp(Tai64(value as u64));
                Ok(UnixTimestamp(tai64_time.0 .0.into()))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl AvroSchemaComponent for UnixTimestamp {
    fn get_schema_in_ctxt(
        _ctxt: &mut HashMap<Name, Schema>,
        _namespace: &Option<String>,
    ) -> Schema {
        // Use Avro's `long` type (i64) for serialization
        Schema::Long
    }
}

impl From<i64> for UnixTimestamp {
    fn from(value: i64) -> Self {
        UnixTimestamp(value.into())
    }
}
