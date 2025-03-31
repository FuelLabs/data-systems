use std::{fmt::Debug, sync::Arc};

use fuel_data_parser::{DataEncoder, DataParserError as EncoderError};
use fuel_streams_subject::subject::{IntoSubject, SubjectPayload};
use fuel_streams_types::{BlockHeight, BlockTimestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum RecordPacketError {
    #[error("Failed to downcast subject")]
    DowncastError,
    #[error("Subject mismatch")]
    SubjectMismatch,
    #[error(transparent)]
    EncodeError(#[from] EncoderError),
    #[error("Failed to decode: {0}")]
    DecodeFailed(String),
}

pub trait PacketBuilder: Send + Sync + 'static {
    type Opts;
    fn build_packets(opts: &Self::Opts) -> Vec<RecordPacket>;
}

pub trait ToPacket: DataEncoder {
    fn to_packet(
        &self,
        subject: &Arc<dyn IntoSubject>,
        block_timestamp: BlockTimestamp,
    ) -> RecordPacket {
        let value = self.encode_json().unwrap_or_else(|_| {
            panic!("Encode failed for {}", std::any::type_name::<Self>())
        });
        RecordPacket::new(
            subject.parse(),
            subject.to_payload(),
            block_timestamp,
            value,
        )
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RecordPointer {
    pub block_height: BlockHeight,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPacket {
    pub value: Vec<u8>,
    pub subject: String,
    pub subject_payload: SubjectPayload,
    pub block_timestamp: BlockTimestamp,
    start_time_timestamp: BlockTimestamp,
    namespace: Option<String>,
}

impl DataEncoder for RecordPacket {}

impl RecordPacket {
    pub fn new(
        subject: impl ToString,
        subject_payload: SubjectPayload,
        block_timestamp: BlockTimestamp,
        value: Vec<u8>,
    ) -> Self {
        let start_time = BlockTimestamp::now();
        Self {
            value,
            subject: subject.to_string(),
            subject_payload,
            block_timestamp,
            start_time_timestamp: start_time,
            namespace: None,
        }
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.to_owned())
    }

    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    pub fn with_start_time(mut self, timestamp: BlockTimestamp) -> Self {
        self.start_time_timestamp = timestamp;
        self
    }

    pub fn subject_id(&self) -> String {
        self.subject_payload.subject.to_string()
    }

    pub fn subject_str(&self) -> String {
        if cfg!(any(test, feature = "test-helpers")) {
            let mut subject = self.subject.to_owned();
            if let Some(namespace) = &self.namespace {
                subject = format!("{}.{}", namespace, subject);
            }
            subject
        } else {
            self.subject.to_owned()
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn calculate_propagation_ms(&self) -> u64 {
        let end_time = BlockTimestamp::now();
        let diff = end_time.diff_ms(&self.start_time_timestamp);
        diff as u64
    }
}
