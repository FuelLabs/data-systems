use actix_ws::{CloseCode, CloseReason, Closed, ProtocolError};
use fuel_streams_core::{stream::StreamError, types::MessagePayloadError};
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
    #[error("Connection closed with reason: {code} - {description}")]
    ClosedWithReason { code: u16, description: String },
    #[error("Subscription failed: {0}")]
    MultSubscription(String),
    #[error(transparent)]
    Encoder(#[from] EncoderError),
    #[error(transparent)]
    Database(#[from] DbError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    SubjectPayload(#[from] SubjectPayloadError),
    #[error(transparent)]
    MessagePayload(#[from] MessagePayloadError),
    #[error("Connection closed")]
    Closed(#[from] Closed),
    #[error("Unsupported message type")]
    UnsupportedMessageType,
    #[error(transparent)]
    ProtocolError(#[from] ProtocolError),
    #[error("Failed to send message")]
    SendError,
    #[error("Client timeout")]
    Timeout,
}

impl From<WebsocketError> for CloseReason {
    fn from(error: WebsocketError) -> Self {
        CloseReason {
            code: match &error {
                // Stream and data handling errors
                WebsocketError::StreamError(_) => CloseCode::Error,
                WebsocketError::MultSubscription(_) => CloseCode::Error,
                WebsocketError::UnserializablePayload(_) => {
                    CloseCode::Unsupported
                }
                WebsocketError::Encoder(_) => CloseCode::Invalid,
                WebsocketError::SubjectPayload(_) => CloseCode::Invalid,
                WebsocketError::MessagePayload(_) => CloseCode::Invalid,

                // Connection state errors
                WebsocketError::ClosedWithReason { code, .. } => {
                    CloseCode::Other(code.to_owned())
                }
                WebsocketError::Closed(_) => CloseCode::Away,

                // Infrastructure errors
                WebsocketError::Database(_) => CloseCode::Error,
                WebsocketError::Store(_) => CloseCode::Error,
                WebsocketError::UnsupportedMessageType => {
                    CloseCode::Unsupported
                }
                WebsocketError::ProtocolError(_) => CloseCode::Protocol,
                WebsocketError::SendError => CloseCode::Error,
                WebsocketError::Timeout => CloseCode::Away,
            },
            description: Some(match &error {
                WebsocketError::StreamError(e) => {
                    format!("Stream error: {}", e)
                }
                WebsocketError::MultSubscription(e) => {
                    format!("Subscription error: {}", e)
                }
                WebsocketError::UnserializablePayload(e) => {
                    format!("Failed to serialize payload: {}", e)
                }
                WebsocketError::ClosedWithReason { description, .. } => {
                    description.clone()
                }
                WebsocketError::Encoder(e) => format!("Encoding error: {}", e),
                WebsocketError::Database(e) => format!("Database error: {}", e),
                WebsocketError::Store(e) => format!("Store error: {}", e),
                WebsocketError::SubjectPayload(e) => {
                    format!("Subject payload error: {}", e)
                }
                WebsocketError::MessagePayload(e) => {
                    format!("Message payload error: {}", e)
                }
                WebsocketError::Closed(_) => {
                    "Connection closed by peer".to_string()
                }
                WebsocketError::UnsupportedMessageType => {
                    "Unsupported message type".to_string()
                }
                WebsocketError::ProtocolError(_) => {
                    "Protocol error".to_string()
                }
                WebsocketError::SendError => {
                    "Failed to send message".to_string()
                }
                WebsocketError::Timeout => "Client timeout".to_string(),
            }),
        }
    }
}

impl From<CloseReason> for WebsocketError {
    fn from(reason: CloseReason) -> Self {
        WebsocketError::ClosedWithReason {
            code: reason.code.into(),
            description: reason.description.unwrap_or_default(),
        }
    }
}
