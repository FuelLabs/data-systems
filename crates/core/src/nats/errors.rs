use async_nats::{
    error,
    jetstream::{context::CreateStreamErrorKind, stream::ConsumerErrorKind},
    ConnectErrorKind,
};
use thiserror::Error;

use super::types::PayloadSize;

#[derive(Error, Debug)]
pub enum NatsError {
    #[error("{subject:?} payload size={payload_size:?} exceeds max_payload_size={max_payload_size:?}")]
    PayloadTooLarge {
        subject: String,
        payload_size: PayloadSize,
        max_payload_size: PayloadSize,
    },

    #[error("Failed to create NATS stream {name}: {source}")]
    CreateStreamFailed {
        name: String,
        #[source]
        source: error::Error<CreateStreamErrorKind>,
    },

    #[error("Failed to create NATS consumer {name}: {source}")]
    CreateConsumerFailed {
        name: String,
        #[source]
        source: error::Error<ConsumerErrorKind>,
    },

    #[error("No NATS key found when connecting to {url}")]
    NkeyNotProvided { url: String },

    #[error("Failed to connect to NATS server at {url}: {source}")]
    ConnectError {
        url: String,
        #[source]
        source: error::Error<ConnectErrorKind>,
    },

    #[error("No valid stream {name} was found no method {method}")]
    NoStreamFound { name: String, method: &'static str },

    #[error("Connection to NATS server at {0} is pending")]
    ConnectionPending(String),

    #[error("Connection to NATS server at {0} is disconnected")]
    ConnectionDisconnected(String),
}
