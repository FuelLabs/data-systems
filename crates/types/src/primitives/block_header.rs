use std::collections::HashMap;

use apache_avro::{
    schema::{derive::AvroSchemaComponent, Name},
    AvroSchema,
    Schema,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use wrapped_int::WrappedU32;

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        // Use Avro's `long` type (i64) for serialization
        Schema::Long
    }
}

// Header type
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
    pub version: BlockHeaderVersion,
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
            FuelCoreBlockHeader::V1(_) => BlockHeaderVersion::V1,
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

// BlockHeaderVersion enum
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Default,
    utoipa::ToSchema,
    derive_more::Display,
    apache_avro::AvroSchema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockHeaderVersion {
    #[default]
    #[display("V1")]
    V1,
}
