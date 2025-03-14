use axum::extract::ws::{CloseFrame, WebSocket};
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
use futures::stream::ReuniteError;
use tokio::task::JoinError;

/// Ws Subscription-related errors
#[derive(Debug, thiserror::Error)]
pub enum WebsocketError {
    #[error("Connection closed with reason: {code} - {description}")]
    ClosedWithReason { code: u16, description: String },
    #[error("Connection closed")]
    Closed(Option<CloseFrame>),
    #[error("Unsupported message type")]
    UnsupportedMessageType,
    #[error("Failed to send message")]
    SendError,
    #[error("Client timeout")]
    Timeout,
    #[error("Subscribe failed: {0}")]
    Subscribe(String),
    #[error("Unsubscribe failed: {0}")]
    Unsubscribe(String),

    #[error(transparent)]
    Axum(#[from] axum::Error),
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
    #[error(transparent)]
    ReuniteError(#[from] ReuniteError<WebSocket, axum::extract::ws::Message>),
}

impl From<WebsocketError> for Option<CloseFrame> {
    fn from(error: WebsocketError) -> Self {
        match &error {
            // Error type (1003)
            WebsocketError::StreamError(_)
            | WebsocketError::JoinHandle(_)
            | WebsocketError::Subscribe(_)
            | WebsocketError::Unsubscribe(_)
            | WebsocketError::Database(_)
            | WebsocketError::Store(_)
            | WebsocketError::SendError
            | WebsocketError::ReuniteError(_) => Some(CloseFrame {
                code: axum::extract::ws::close_code::UNSUPPORTED,
                reason: error.to_string().into(),
            }),

            // Invalid type (1007)
            WebsocketError::Encoder(_)
            | WebsocketError::SubjectPayload(_)
            | WebsocketError::MessagePayload(_) => Some(CloseFrame {
                code: axum::extract::ws::close_code::INVALID,
                reason: error.to_string().into(),
            }),

            // Unsupported type (1003)
            WebsocketError::Serde(_)
            | WebsocketError::ServerRequest(_)
            | WebsocketError::UnsupportedMessageType => Some(CloseFrame {
                code: axum::extract::ws::close_code::UNSUPPORTED,
                reason: error.to_string().into(),
            }),

            // Away type (1001)
            WebsocketError::Closed(_) | WebsocketError::Timeout => {
                Some(CloseFrame {
                    code: axum::extract::ws::close_code::AWAY,
                    reason: error.to_string().into(),
                })
            }

            // Other types
            WebsocketError::ClosedWithReason { code, description } => {
                Some(CloseFrame {
                    code: *code,
                    reason: description.clone().into(),
                })
            }
            WebsocketError::Axum(_) => Some(CloseFrame {
                code: axum::extract::ws::close_code::UNSUPPORTED,
                reason: error.to_string().into(),
            }),
            WebsocketError::Subjects(_) => Some(CloseFrame {
                code: axum::extract::ws::close_code::UNSUPPORTED,
                reason: error.to_string().into(),
            }),
            WebsocketError::RecordEntity(_) => Some(CloseFrame {
                code: axum::extract::ws::close_code::UNSUPPORTED,
                reason: error.to_string().into(),
            }),
        }
    }
}

impl From<CloseFrame> for WebsocketError {
    fn from(frame: CloseFrame) -> Self {
        WebsocketError::ClosedWithReason {
            code: frame.code,
            description: frame.reason.to_string(),
        }
    }
}
