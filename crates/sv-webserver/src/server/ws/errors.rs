use fuel_streams_core::StreamError;
use fuel_streams_store::{
    db::DbError,
    record::EncoderError,
    store::StoreError,
};

/// Ws Subscription-related errors
#[derive(Debug, thiserror::Error)]
pub enum WsSubscriptionError {
    #[error("Unknown subject name: {0}")]
    UnknownSubjectName(String),
    #[error("Unsupported wildcard pattern: {0}")]
    UnsupportedWildcardPattern(String),
    #[error(transparent)]
    UnserializablePayload(#[from] serde_json::Error),
    #[error(transparent)]
    Stream(#[from] StreamError),
    #[error("Closed by client with reason: {0}")]
    ClosedWithReason(String),
    #[error(transparent)]
    Encoder(#[from] EncoderError),
    #[error(transparent)]
    Database(#[from] DbError),
    #[error(transparent)]
    Store(#[from] StoreError),
}
