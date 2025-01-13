use actix_ws::Closed;
use fuel_streams_core::stream::StreamError;
use fuel_streams_domains::SubjectPayloadError;
use fuel_streams_store::{
    db::DbError,
    record::EncoderError,
    store::StoreError,
};

/// Ws Subscription-related errors
#[derive(Debug, thiserror::Error)]
pub enum WebsocketError {
    #[error("Stream error: {0}")]
    StreamError(#[from] StreamError),
    #[error("Unserializable payload: {0}")]
    UnserializablePayload(#[from] serde_json::Error),
    #[error("Connection closed: {0}")]
    ClosedWithReason(String),
    #[error(transparent)]
    Encoder(#[from] EncoderError),
    #[error(transparent)]
    Database(#[from] DbError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    SubjectPayload(#[from] SubjectPayloadError),
    #[error("Connection closed")]
    Closed(#[from] Closed),
}
