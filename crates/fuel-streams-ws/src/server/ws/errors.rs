use displaydoc::Display as DisplayDoc;
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
}
