use displaydoc::Display as DisplayDoc;
use fuel_streams_core::StreamError;
use fuel_streams_store::{
    db::DbError,
    record::EncoderError,
    store::StoreError,
};
use thiserror::Error;

/// Ws Subscription-related errors
#[derive(Debug, DisplayDoc, Error)]
pub enum WsSubscriptionError {
    /// Unknown subject name: `{0}`
    UnknownSubjectName(String),
    /// Unsupported wildcard pattern: `{0}`
    UnsupportedWildcardPattern(String),
    /// Unserializable payload: `{0}`
    UnserializablePayload(#[from] serde_json::Error),
    /// Stream Error: `{0}`
    Stream(#[from] StreamError),
    /// Closed by client with reason: `{0}`
    ClosedWithReason(String),
    /// Failed to encode or decode data: {0}
    Encoder(#[from] EncoderError),
    /// Failed with Database: {0}
    Database(#[from] DbError),
    /// Failed on store: {0}
    Store(#[from] StoreError),
}
