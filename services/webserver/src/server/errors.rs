use actix_ws::{CloseCode, CloseReason, Closed, ProtocolError};
use fuel_streams_core::{
    prelude::SubjectPayloadError,
    stream::StreamError,
    types::{MessagePayloadError, ServerRequestError},
};
use fuel_streams_domains::SubjectsError;
use fuel_streams_store::{
    db::DbError,
    record::{EncoderError, RecordEntityError},
    store::StoreError,
};
use tokio::task::JoinError;

/// Ws Subscription-related errors
#[derive(Debug, thiserror::Error)]
pub enum WebsocketError {
    #[error("Connection closed with reason: {code} - {description}")]
    ClosedWithReason { code: u16, description: String },
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
    #[error("Subscribe failed: {0}")]
    Subscribe(String),
    #[error("Unsubscribe failed: {0}")]
    Unsubscribe(String),

    #[error(transparent)]
    JoinHandle(#[from] JoinError),
    #[error(transparent)]
    ServerRequest(#[from] ServerRequestError),
    #[error(transparent)]
    StreamError(#[from] StreamError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
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
    #[error(transparent)]
    Subjects(#[from] SubjectsError),
    #[error(transparent)]
    RecordEntity(#[from] RecordEntityError),
}

impl From<WebsocketError> for CloseReason {
    fn from(error: WebsocketError) -> Self {
        CloseReason {
            code: match &error {
                // Error type
                WebsocketError::StreamError(_)
                | WebsocketError::JoinHandle(_)
                | WebsocketError::Subscribe(_)
                | WebsocketError::Unsubscribe(_)
                | WebsocketError::Database(_)
                | WebsocketError::Store(_)
                | WebsocketError::SendError => CloseCode::Error,

                // Invalid type
                WebsocketError::Encoder(_)
                | WebsocketError::SubjectPayload(_)
                | WebsocketError::MessagePayload(_) => CloseCode::Invalid,

                // Unsupported type
                WebsocketError::Serde(_)
                | WebsocketError::ServerRequest(_)
                | WebsocketError::UnsupportedMessageType => {
                    CloseCode::Unsupported
                }

                // Away type
                WebsocketError::Closed(_) | WebsocketError::Timeout => {
                    CloseCode::Away
                }

                // Other types
                WebsocketError::ClosedWithReason { code, .. } => {
                    CloseCode::Other(code.to_owned())
                }
                WebsocketError::ProtocolError(_) => CloseCode::Protocol,
                WebsocketError::Subjects(_) => CloseCode::Error,
                WebsocketError::RecordEntity(_) => CloseCode::Error,
            },
            description: Some(error.to_string()),
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
