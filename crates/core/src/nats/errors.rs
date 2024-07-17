use async_nats::jetstream::context::CreateStreamErrorKind;
use async_nats::jetstream::stream::ConsumerErrorKind;
use async_nats::{error, ConnectErrorKind};
use thiserror::Error;

use super::types::PayloadSize;
use super::Subject;

#[derive(Error, Debug)]
pub enum NatsError {
    #[error("{subject:?} payload size={payload_size:?} exceeds max_payload_size={max_payload_size:?}")]
    PayloadTooLarge {
        subject: Subject,
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

    #[error("Failed to connect to NATS server at {url}: {source}")]
    ConnectError {
        url: String,
        #[source]
        source: error::Error<ConnectErrorKind>,
    },

    #[error("Connection to NATS server at {0} is pending")]
    ConnectionPending(String),

    #[error("Connection to NATS server at {0} is disconnected")]
    ConnectionDisconnected(String),
}
