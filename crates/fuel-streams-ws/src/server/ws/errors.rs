use displaydoc::Display as DisplayDoc;
use fuel_streams_core::StreamError;
use thiserror::Error;

/// Ws Subscription-related errors
#[derive(Debug, DisplayDoc, Error)]
pub enum WsSubscriptionError {
    /// Unparsable subscription payload: `{0}`
    UnparsablePayload(serde_json::Error),
    /// Unknown subject name: `{0}`
    UnknownSubjectName(String),
    /// Unsupported wildcard pattern: `{0}`
    UnsupportedWildcardPattern(String),
    /// Unserializable message payload: `{0}`
    UnserializableMessagePayload(serde_json::Error),
    /// Stream Error: `{0}`
    Stream(#[from] StreamError),
    /// Closed by client with reason: `{0}`
    ClosedWithReason(String),
}
